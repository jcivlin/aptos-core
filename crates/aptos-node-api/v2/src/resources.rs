// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use async_graphql::{Json, SimpleObject, Value};

#[derive(Clone, Debug, SimpleObject)]
pub struct Resource {
    json_data_v1: Json<Value>,
}

impl Resource {
    pub fn new(json_data_v1: Json<Value>) -> Self {
        Self { json_data_v1 }
    }
}
