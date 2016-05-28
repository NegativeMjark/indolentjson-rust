/* Copyright 2016 Mark Haines
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#![cfg_attr(all(test, feature = "quickcheck_test"), feature(plugin))]
#![cfg_attr(all(test, feature = "quickcheck_test"), plugin(quickcheck_macros))]

pub mod compact;
pub mod readhex;
pub mod parse;
pub mod validate;
pub mod strings;

#[cfg(all(test, feature = "quickcheck_test"))]
extern crate quickcheck;
