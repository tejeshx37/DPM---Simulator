use super::{Closed, Open, Project, State, Untitled, Workspace};
use directories::BaseDirs;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::{
    ffi::OsString,
    fs,
    hash::Hash,
    io::{self, ErrorKind},
    mem,
    ops::{Index, IndexMut},
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OpenHandle(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct UntitledHandle(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ClosedHandle(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RecentHandle(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkspaceHandle(u64);

impl WorkspaceHandle {
    fn for_workspace(workspace: &Workspace) -> Self {
        Self(fxhash::hash64(workspace.path()))
    }
}

macro_rules! impl_for_project {
    ( $( $handle:ty ),* ) => {
        $( impl $handle {
            fn for_project<S: State>(project: &Project<S>) -> Self {
                Self(fxhash::hash64(project))
            }
        } )*
    };
}

impl_for_project!(OpenHandle, ClosedHandle);

type ProjectMap<H, S> = IndexMap<H, Project<S>>;

#[derive(Debug, Serialize)]
pub struct Manager<D> {
    home_dir: PathBuf,
    workspace: Workspace,
    open_projects: ProjectMap<OpenHandle, Open<D>>,
    untitled_projects: Vec<Project<Untitled<D>>>,
    #[serde(skip_serializing)]
    closed_projects: ProjectMap<ClosedHandle, Closed>,
    recent_projects: ProjectMap<RecentHandle, Closed>,
    recent_workspaces: IndexMap<WorkspaceHandle, Workspace>,
}

fn workspace_projects(workspace: &Workspace) -> io::Result<ProjectMap<ClosedHandle, Closed>> {
    puffin::profile_function!();
    fs::read_dir(workspace.path())?
        .filter_map(|result| {
            result
                .and_then(|entry| {
                    super::has_valid_extension(&entry.path())
                        .then(|| {
                            let project = Project {
                                name: super::name_from_project_path(&entry.path())?,
                                workspace_path: super::workspace_path_from_project_path(
                                    &entry.path(),
                                )?,
                                state: Closed,
                            };
                            Ok((ClosedHandle::for_project(&project), project))
                        })
                        .transpose()
                })
                .transpose()
        })
        .collect()
}

macro_rules! delete_project {
    ($( $type:ident ),*) => {
        paste::paste! {
            $( pub fn [<delete_ $type:lower _project>](&mut self, handle: &[<$type Handle>]) -> io::Result<()> {
                puffin::profile_function!();
                let Some(project) = self.[<$type:lower _projects>].shift_remove(handle) else {
                    return Err(io::Error::new(
                        ErrorKind::Other,
                        "Handle points to a non existent project",
                    ));
                };
                match project.delete() {
                    Ok(()) => Ok(()),
                    Err((project, err)) => {
                        self.[<$type:lower _projects>].insert(*handle, project);
                        Err(err)
                    }
                }
            } )*
        }
    };
}

impl<D> Manager<D>
where
    D: Default + Serialize + for<'de> Deserialize<'de>,
{
    pub fn default() -> io::Result<Self> {
        puffin::profile_function!();
        let home_dir = BaseDirs::new()
            .map(|dirs| dirs.home_dir().to_path_buf())
            .ok_or_else(|| io::Error::new(ErrorKind::Other, "Unable to get user home"))?;
        Workspace::default(&home_dir).and_then(|workspace| {
            Ok(Self {
                home_dir,
                open_projects: ProjectMap::default(),
                untitled_projects: Vec::default(),
                closed_projects: workspace_projects(&workspace)?,
                recent_projects: ProjectMap::default(),
                workspace,
                recent_workspaces: IndexMap::default(),
            })
        })
    }

    pub fn open_project<P>(&mut self, path: P) -> io::Result<OpenHandle>
    where
        P: AsRef<Path>,
    {
        puffin::profile_function!();
        let name = super::name_from_project_path(path.as_ref())?;
        let workspace = super::workspace_path_from_project_path(path.as_ref())?;
        let project = super::storage::read_data_in_path(path).map(|state| Project {
            name,
            workspace_path: workspace,
            state,
        })?;
        let handle = OpenHandle::for_project(&project);
        self.recent_projects
            .insert(RecentHandle(handle.0), project.closed());
        self.open_projects.insert(handle, project);
        Ok(handle)
    }

    delete_project!(Open, Closed);

    pub fn clear_recents(&mut self) {
        self.recent_projects.clear();
        self.recent_workspaces.clear();
    }
}

impl<D> Manager<D>
where
    D: Default + Serialize + for<'de> Deserialize<'de>,
{
    pub fn workspace(&self) -> &Workspace {
        &self.workspace
    }

    pub fn set_workspace(&mut self, path: PathBuf) -> io::Result<()> {
        puffin::profile_function!();
        let workspace = Workspace::new(&self.home_dir, path)?;
        self.closed_projects = workspace_projects(&workspace)?;
        let workspace = mem::replace(&mut self.workspace, workspace);
        self.recent_workspaces
            .insert(WorkspaceHandle::for_workspace(&workspace), workspace);
        Ok(())
    }

    pub fn refresh_workspace(&mut self) -> io::Result<()> {
        puffin::profile_function!();
        let mut closed_projects = workspace_projects(&self.workspace)?;
        closed_projects.retain(|handle, _| !self.open_projects.contains_key(&OpenHandle(handle.0)));
        self.closed_projects = closed_projects;
        Ok(())
    }

    pub fn has_recent_workspaces(&self) -> bool {
        !self.recent_workspaces.is_empty()
    }

    pub fn recent_workspaces(&self) -> impl Iterator<Item = &WorkspaceHandle> + '_ {
        self.recent_workspaces.keys().rev()
    }

    pub fn open_recent_workspace(&mut self, handle: &WorkspaceHandle) -> io::Result<()> {
        puffin::profile_function!();
        let Some(workspace) = self.recent_workspaces.shift_remove(handle) else {
            return Err(io::Error::new(
                ErrorKind::NotFound,
                "Handle points to a non existant workspace",
            ));
        };
        self.closed_projects = workspace_projects(&workspace)?;
        self.workspace = workspace;
        Ok(())
    }
}

impl<D> Index<&WorkspaceHandle> for Manager<D> {
    type Output = Workspace;

    fn index(&self, index: &WorkspaceHandle) -> &Self::Output {
        &self.recent_workspaces[index]
    }
}

macro_rules! handles_getter {
    ($type:ident) => {
        paste::paste! {
            pub fn [<$type:lower _project_handles>](&self) -> impl Iterator<Item = &[<$type Handle>]> + '_ {
                self.[<$type:lower _projects>].keys().rev()
            }
        }
    };
}

macro_rules! impl_index {
    ($type:ident, $output:ty) => {
        paste::paste! {
            impl<D> Index<&[<$type Handle>]> for Manager<D> {
                type Output = $output;

                fn index(&self, index: &[<$type Handle>]) -> &Self::Output {
                    &self.[<$type:lower _projects>][index]
                }
            }
        }
    };
}

macro_rules! get_mut {
    ($type:ident) => {
        paste::paste! {
            pub fn [<get_ $type:lower _project_mut>](&mut self, handle: &[<$type Handle>]) -> Option<&mut Project<$type<D>>> {
                self.[<$type:lower _projects>].get_mut(handle)
            }
        }
    };
}

macro_rules! has_projects {
    ($type:ident) => {
        paste::paste! {
            pub fn [<has_ $type:lower _projects>](&self) -> bool {
                !self.[<$type:lower _projects>].is_empty()
            }
        }
    };
}

impl<D> Manager<D>
where
    D: Default + Serialize + for<'de> Deserialize<'de>,
{
    has_projects!(Untitled);

    pub fn untitled_project_handles(&self) -> impl Iterator<Item = UntitledHandle> {
        (0..self.untitled_projects.len()).map(UntitledHandle)
    }

    pub fn create_untitled_project(&mut self) {
        puffin::profile_function!();
        let project = Project {
            name: OsString::from("Untitled"),
            workspace_path: self.workspace.path().to_owned(),
            state: Untitled(D::default()),
        };
        self.untitled_projects.push(project);
    }

    pub fn discard_untitled_project(&mut self, handle: &UntitledHandle) {
        puffin::profile_function!();
        self.untitled_projects.remove(handle.0);
    }

    pub fn save_untitled_project(
        &mut self,
        handle: &UntitledHandle,
        path: PathBuf,
    ) -> io::Result<OpenHandle> {
        puffin::profile_function!();
        if self.untitled_projects.is_empty() || handle.0 >= self.untitled_projects.len() {
            return Err(io::Error::new(
                ErrorKind::Other,
                "Handle points to a non existent project",
            ));
        }
        let project = self.untitled_projects.remove(handle.0);
        match project.set_path(path) {
            Ok(mut project) => {
                let save_result = project.save();
                let handle = OpenHandle::for_project(&project);
                self.open_projects.insert(handle, project);
                save_result.map(|()| handle)
            }
            Err((project, err)) => {
                self.untitled_projects
                    .insert(handle.0.checked_sub(1).unwrap_or_default(), project);
                Err(err)
            }
        }
    }
}

impl<D> Index<UntitledHandle> for Manager<D> {
    type Output = Project<Untitled<D>>;

    fn index(&self, index: UntitledHandle) -> &Self::Output {
        &self.untitled_projects[index.0]
    }
}

impl<D> IndexMut<UntitledHandle> for Manager<D> {
    fn index_mut(&mut self, index: UntitledHandle) -> &mut Self::Output {
        &mut self.untitled_projects[index.0]
    }
}

impl<D> Manager<D>
where
    D: Default + Serialize + for<'de> Deserialize<'de>,
{
    handles_getter!(Open);
    has_projects!(Open);

    pub fn save_open_project_in_path(
        &mut self,
        handle: &OpenHandle,
        path: PathBuf,
    ) -> io::Result<OpenHandle> {
        puffin::profile_function!();
        let mut project = self.open_projects.shift_remove(handle).ok_or_else(|| {
            io::Error::new(
                ErrorKind::Other,
                "Handle does not point to an existing open project",
            )
        })?;
        project.save_at_path(path)?;
        let handle = OpenHandle::for_project(&project);
        self.open_projects.insert(handle, project);
        Ok(handle)
    }

    pub fn close_open_project(&mut self, handle: &OpenHandle) {
        puffin::profile_function!();
        let Some(project) = self.open_projects.shift_remove(handle) else {
            return;
        };
        if project.workspace_path != self.workspace.path() {
            return;
        }
        self.closed_projects
            .insert(ClosedHandle(handle.0), project.close());
    }
}

impl_index!(Open, Project<Open<D>>);

impl<D> Manager<D>
where
    D: Default + Serialize + for<'de> Deserialize<'de>,
{
    handles_getter!(Recent);
    get_mut!(Open);
    has_projects!(Recent);

    pub fn open_recent_project(&mut self, handle: &RecentHandle) -> io::Result<OpenHandle> {
        puffin::profile_function!();
        let Some(project) = self.recent_projects.shift_remove(handle) else {
            return Err(io::Error::new(
                ErrorKind::Other,
                "Handle points to a non existent project",
            ));
        };
        let open_handle = OpenHandle(handle.0);
        if self.open_projects.contains_key(&open_handle) {
            return Ok(open_handle);
        }
        match project.open() {
            Ok(project) => {
                self.recent_projects.insert(*handle, project.closed());
                self.open_projects.insert(open_handle, project);
                Ok(open_handle)
            }
            Err((_, err)) => Err(err),
        }
    }
}

impl_index!(Recent, Project<Closed>);

impl<D> Manager<D>
where
    D: Default + Serialize + for<'de> Deserialize<'de>,
{
    handles_getter!(Closed);
    has_projects!(Closed);

    pub fn open_closed_project(&mut self, handle: &ClosedHandle) -> io::Result<OpenHandle> {
        puffin::profile_function!();
        let Some(project) = self.closed_projects.shift_remove(handle) else {
            return Err(io::Error::new(
                ErrorKind::Other,
                "Handle points to a non existent project",
            ));
        };
        match project.open() {
            Ok(project) => {
                let handle = OpenHandle(handle.0);
                self.open_projects.insert(handle, project);
                Ok(handle)
            }
            Err((project, err)) => {
                self.closed_projects.insert(*handle, project);
                Err(err)
            }
        }
    }
}

impl_index!(Closed, Project<Closed>);

mod serde_impl {
    use super::*;
    use serde::{
        de::{self, MapAccess, Visitor},
        Deserialize, Deserializer,
    };
    use std::{
        fmt::{self, Formatter},
        marker::PhantomData,
    };
    use strum::{AsRefStr, VariantNames};

    #[derive(Deserialize, VariantNames, AsRefStr)]
    #[serde(field_identifier, rename_all = "snake_case")]
    #[strum(serialize_all = "snake_case")]
    enum Field {
        HomeDir,
        Workspace,
        OpenProjects,
        UntitledProjects,
        RecentProjects,
        RecentWorkspaces,
    }

    struct ManagerVisitor<D> {
        _marker: PhantomData<D>,
    }

    impl<'de, D> Visitor<'de> for ManagerVisitor<D>
    where
        D: Deserialize<'de>,
    {
        type Value = Manager<D>;

        fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
            formatter.write_str("struct Manager")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let mut home_dir = None;
            let mut workspace = None;
            let mut open_projects = None;
            let mut untitled_projects = None;
            let mut recent_projects = None;
            let mut recent_workspaces = None;
            while let Some(key) = map.next_key()? {
                match key {
                    Field::HomeDir => {
                        if home_dir.is_some() {
                            return Err(de::Error::duplicate_field(Field::HomeDir.as_ref()));
                        }
                        home_dir = Some(map.next_value()?);
                    }
                    Field::Workspace => {
                        if workspace.is_some() {
                            return Err(de::Error::duplicate_field(Field::Workspace.as_ref()));
                        }
                        workspace = Some(map.next_value()?);
                    }
                    Field::OpenProjects => {
                        if open_projects.is_some() {
                            return Err(de::Error::duplicate_field(Field::OpenProjects.as_ref()));
                        }
                        open_projects = Some(map.next_value()?);
                    }
                    Field::UntitledProjects => {
                        if untitled_projects.is_some() {
                            return Err(de::Error::duplicate_field(
                                Field::UntitledProjects.as_ref(),
                            ));
                        }
                        untitled_projects = Some(map.next_value()?);
                    }
                    Field::RecentProjects => {
                        if recent_projects.is_some() {
                            return Err(de::Error::duplicate_field(Field::RecentProjects.as_ref()));
                        }
                        recent_projects = Some(map.next_value()?);
                    }
                    Field::RecentWorkspaces => {
                        if recent_workspaces.is_some() {
                            return Err(de::Error::duplicate_field(
                                Field::RecentWorkspaces.as_ref(),
                            ));
                        }
                        recent_workspaces = Some(map.next_value()?);
                    }
                }
            }
            let home_dir: PathBuf =
                home_dir.ok_or_else(|| de::Error::missing_field(Field::HomeDir.as_ref()))?;
            let mut workspace: Workspace =
                workspace.ok_or_else(|| de::Error::missing_field(Field::Workspace.as_ref()))?;
            let open_projects: ProjectMap<OpenHandle, Open<D>> = open_projects
                .ok_or_else(|| de::Error::missing_field(Field::OpenProjects.as_ref()))?;
            let untitled_projects = untitled_projects
                .ok_or_else(|| de::Error::missing_field(Field::UntitledProjects.as_ref()))?;
            let recent_projects = recent_projects
                .ok_or_else(|| de::Error::missing_field(Field::RecentProjects.as_ref()))?;
            let mut recent_workspaces: IndexMap<WorkspaceHandle, Workspace> =
                recent_workspaces.unwrap_or_default();
            recent_workspaces.retain(|_, workspace| workspace.path().is_dir());

            if !workspace.path().is_dir() {
                workspace = Workspace::default(&home_dir).map_err(de::Error::custom)?;
            }

            let mut closed_projects = workspace_projects(&workspace).map_err(de::Error::custom)?;
            closed_projects.retain(|handle, _| !open_projects.contains_key(&OpenHandle(handle.0)));

            Ok(Manager {
                home_dir,
                workspace,
                open_projects,
                untitled_projects,
                closed_projects,
                recent_projects,
                recent_workspaces,
            })
        }
    }

    impl<'de, Data> Deserialize<'de> for Manager<Data>
    where
        Data: Deserialize<'de>,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_struct(
                "Manager",
                Field::VARIANTS,
                ManagerVisitor {
                    _marker: PhantomData,
                },
            )
        }
    }
}
