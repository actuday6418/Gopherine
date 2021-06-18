use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct File {
    name: String,
    link: String,
}

impl File {
    fn new(name: String, link: String) -> File {
        File {
            name: name,
            link: link,
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_link(&self) -> &str {
        &self.link
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Directory {
    name: String,
    children_d: Vec<Directory>,
    children_f: Vec<File>,
}

impl Directory {
    pub fn new(name: &str) -> Directory {
        Directory {
            name: String::from(name),
            children_d: Vec::new(),
            children_f: Vec::new(),
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Serialize, Deserialize)]
pub struct CourseTree {
    root: Directory,
}

impl CourseTree {
    pub fn new() -> Self {
        CourseTree {
            root: Directory::new("root"),
        }
    }
    fn traverse_course_tree(
        root: &mut Directory,
        change_directory: Vec<String>,
    ) -> std::option::Option<&mut Directory> {
        let mut req_directory = root;
        for next_dir in change_directory {
            req_directory = match req_directory
                .children_d
                .iter_mut()
                .find(|dir| dir.name == next_dir)
            {
                Some(d) => d,
                None => return None,
            }
        }
        Some(req_directory)
    }

    pub fn add_directory(
        &mut self,
        change_directory: Vec<String>,
        name: &String,
    ) -> std::result::Result<(), ()> {
        match CourseTree::traverse_course_tree(&mut self.root, change_directory) {
            Some(dir) => dir,
            None => return Err(()),
        }
        .children_d
        .push(Directory::new(name.as_str()));
        Ok(())
    }

    pub fn add_file(
        &mut self,
        change_directory: Vec<String>,
        name: &String,
        link: &String,
    ) -> std::result::Result<(), ()> {
        match CourseTree::traverse_course_tree(&mut self.root, change_directory) {
            Some(dir) => dir,
            None => return Err(()),
        }
        .children_f
        .push(File::new(name.clone(), link.clone()));
        Ok(())
    }

    pub fn directory_contents(
        &mut self,
        change_directory: Vec<String>,
    ) -> std::option::Option<(Vec<Directory>, Vec<File>)> {
        let req_dir = match CourseTree::traverse_course_tree(&mut self.root, change_directory) {
            Some(dir) => dir,
            None => return None,
        };
        Some((req_dir.children_d.clone(), req_dir.children_f.clone()))
    }
}
