use std::{collections::BTreeMap, process::Command};

pub struct CargoOpts {
    package: String,
    action: String,
    release: bool,
    build_std: bool,
    features: Vec<String>,
    env: BTreeMap<String, String>,
    target: Option<String>,
}

impl CargoOpts {
    pub fn new(package: String) -> Self {
        Self {
            package,
            action: "build".into(),
            release: false,
            build_std: false,
            features: Vec::new(),
            env: BTreeMap::new(),
            target: None,
        }
    }
}

impl CargoOpts {
    pub fn release(&mut self) -> &mut Self {
        self.release = true;
        self
    }

    pub fn target(&mut self, target: String) -> &mut Self {
        self.target = Some(target);
        self
    }

    pub fn action<S: AsRef<str>>(&mut self, action: S) -> &mut Self {
        self.action = action.as_ref().to_string();
        self
    }

    pub fn build_std(&mut self) -> &mut Self {
        self.build_std = true;
        self
    }

    pub fn feature<S: AsRef<str>>(&mut self, feature: S) -> &mut Self {
        self.features.push(feature.as_ref().to_string());
        self
    }

    pub fn env<S1: AsRef<str>, S2: AsRef<str>>(&mut self, name: S1, value: S2) -> &mut Self {
        self.env.insert(name.as_ref().into(), value.as_ref().into());
        self
    }
}

impl CargoOpts {
    pub fn done(&mut self) {
        let mut cargo = Command::new(env!("CARGO"));
        cargo.arg(&self.action);
        cargo.arg("-p").arg(&self.package);

        if self.release {
            cargo.arg("--release");
        }
        if self.build_std {
            cargo.arg("-Zbuild-std");
        }
        if let Some(target) = &self.target {
            cargo.arg("--target").arg(target);
        }

        cargo.arg("--features").arg(self.features.join(","));

        for (name, value) in self.env.iter() {
            cargo.env(name, value);
        }

        cargo.status().unwrap().exit_ok().unwrap();
    }
}
