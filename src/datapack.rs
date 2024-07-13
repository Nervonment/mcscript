use std::{
    fs::{create_dir, remove_dir_all, File},
    io::{Result, Write},
};

pub struct Datapack {
    name: String,
    namespaces: Vec<Namespace>,
}

pub struct Namespace {
    name: String,
    mcfunctions: Vec<Mcfunction>,
}

pub struct Mcfunction {
    name: String,
    content: String,
}

impl Datapack {
    pub fn new(name: String) -> Self {
        Self {
            name,
            namespaces: vec![],
        }
    }

    pub fn write_to_file(&self) -> Result<()> {
        match create_dir(&self.name) {
            Ok(()) => {}
            Err(err) => match err.kind() {
                std::io::ErrorKind::AlreadyExists => {
                    remove_dir_all(&self.name)?;
                    create_dir(&self.name)?;
                }
                _ => {}
            },
        };
        let pack_mcmeta = [&self.name, "pack.mcmeta"].join("/");
        create_dir([&self.name, "data"].join("/"))?;
        let mut pack_mcmeta = File::create(pack_mcmeta)?;
        pack_mcmeta.write_all("{\n    \"pack\": {\n        \"description\": \"Generated from mcscript.\",\n        \"pack_format\": 48\n    }\n}".as_bytes())?;

        for namespace in &self.namespaces {
            namespace.write_to_file(&self.name)?;
        }

        Ok(())
    }

    pub fn append_namespace(&mut self, namespace: Namespace) {
        self.namespaces.push(namespace);
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Namespace {
    pub fn new(name: String) -> Self {
        Self {
            name,
            mcfunctions: vec![],
        }
    }

    pub fn write_to_file(&self, pack_name: &str) -> Result<()> {
        create_dir([pack_name, "data", &self.name].join("/"))?;
        create_dir([pack_name, "data", &self.name, "function"].join("/"))?;
        for mcfunction in &self.mcfunctions {
            let mut file_name = mcfunction.name.clone();
            file_name.push_str(".mcfunction");
            let mut file =
                File::create([pack_name, "data", &self.name, "function", &file_name].join("/"))?;
            file.write_all(mcfunction.content.as_bytes())?;
        }
        Ok(())
    }

    pub fn append_mcfunction(&mut self, mcfunction: Mcfunction) {
        self.mcfunctions.push(mcfunction);
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Mcfunction {
    pub fn new(name: String) -> Self {
        Self {
            name,
            content: "".into(),
        }
    }

    pub fn append_command(&mut self, command: &str) {
        self.content.push_str(command);
        self.content.push_str("\n");
    }

    pub fn append_commands(&mut self, commands: Vec<&str>) {
        self.content.push_str(&commands.join("\n"));
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
