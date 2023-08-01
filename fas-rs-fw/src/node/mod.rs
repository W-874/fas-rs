/* Copyright 2023 shadow3aaa@gitbub.com
*
*  Licensed under the Apache License, Version 2.0 (the "License");
*  you may not use this file except in compliance with the License.
*  You may obtain a copy of the License at
*
*      http://www.apache.org/licenses/LICENSE-2.0
*
*  Unless required by applicable law or agreed to in writing, software
*  distributed under the License is distributed on an "AS IS" BASIS,
*  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
*  See the License for the specific language governing permissions and
*  limitations under the License. */
mod single;

use log::error;
use parking_lot::RwLock;
pub use single::NODE;

use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, prelude::*},
    path::Path,
    process,
    sync::Arc,
    thread,
};

use log::debug;
use unix_named_pipe;

const NODE_PATH: &str = "/cache/fas_rs_nodes";

pub struct Node(RwLock<HashMap<&'static str, Arc<RwLock<String>>>>);

impl Node {
    pub(crate) fn init() -> Result<Self, io::Error> {
        let _ = fs::remove_dir_all(NODE_PATH);
        fs::create_dir(NODE_PATH)?;
        let id_value = HashMap::new();
        Ok(Self(id_value.into()))
    }

    /// 创建一个新节点
    ///
    /// # Errors
    ///
    /// 创建错误
    ///
    /// # Panics
    ///
    /// 创建线程错误/节点被删除
    pub fn create_node(&self, id: &'static str, default: &str) -> Result<(), io::Error> {
        if self.0.read().contains_key(id) {
            return Ok(());
        }

        let path = Path::new(NODE_PATH).join(id);

        unix_named_pipe::create(&path, Some(0o644))?;

        let value = Arc::new(RwLock::new(default.to_string()));
        self.0.write().insert(id, value.clone());

        thread::Builder::new()
            .name("NodeWatcher".into())
            .spawn(move || {
                let mut retry_count = 0;
                loop {
                    if retry_count > 10 {
                        error!("Too many read config retries");
                        process::exit(1);
                    }

                    let Ok(mut file) = File::open(&path) else {
                        retry_count += 1;
                        continue;
                    };

                    let mut buffer = String::new();
                    if file.read_to_string(&mut buffer).is_err() {
                        retry_count += 1;
                        continue;
                    }

                    let buffer = buffer.trim().lines().last().unwrap_or_default();

                    debug!("Recv node value update: {} {buffer}", path.display());

                    *value.write() = buffer.to_string();

                    retry_count = 0;
                }
            })
            .unwrap();

        Ok(())
    }

    /// 读取指定的节点
    ///
    /// # Errors
    ///
    /// 节点未创建/不存在
    #[inline]
    pub fn read_node(&self, id: &'static str) -> Result<String, &'static str> {
        Ok((*self.0.read().get(id).ok_or("No such a node")?.read()).clone())
    }
}