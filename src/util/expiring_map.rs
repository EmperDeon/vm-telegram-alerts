// MIT License
//
// Copyright (c) 2018 Josh Mcguigan
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
//
// https://github.com/JoshMcguigan/expiring_map

use std::time::{SystemTime, Duration};
use std::collections::HashMap;
use std::hash::Hash;
use std::borrow::Borrow;
use std::ops::Add;

struct ValueContainer<V> {
  value: V,
  expire_time: SystemTime,
}

pub(crate) struct ExpiringMap<K, V> {
  inner: HashMap<K, ValueContainer<V>>,
  time_to_live: Duration,
}

impl<V> ValueContainer<V> {
  fn new(value: V, expire_time: SystemTime) -> Self {
    ValueContainer {
      value,
      expire_time,
    }
  }
}

impl<K, V> ExpiringMap<K, V>
  where K: Eq + Hash
{
  pub(crate) fn new(time_to_live: Duration) -> Self {
    ExpiringMap {
      inner: HashMap::new(),
      time_to_live,
    }
  }

  pub(crate) fn insert(&mut self, k: K, v: V, current_time: SystemTime) -> Option<V> {
    let value_container =
      ValueContainer::new(v, current_time.add(self.time_to_live));

    self.inner.insert(k, value_container)
      .and_then(|val_container| {
        if current_time.le(&val_container.expire_time) {
          // only return the previous value if it was not expired
          Some(val_container.value)
        } else {
          None
        }
      })
  }

  pub(crate) fn get<Q: ?Sized>(&mut self, k: &Q, current_time: SystemTime) -> Option<&V>
    where K: Borrow<Q>,
          Q: Hash + Eq
  {
    self.inner.get(k).and_then(|val_container| {
      if current_time.le(&val_container.expire_time) {
        Some(&val_container.value)
      } else {
        None
      }
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn get_test_map() -> ExpiringMap<String, String> {
    let time_to_live = Duration::from_secs(60);

    ExpiringMap::new(time_to_live)
  }

  #[test]
  fn insert_and_get() {
    let mut map = get_test_map();

    map.insert("keyA".to_owned(), "valA".to_owned(), SystemTime::now());

    assert_eq!(Some(&mut "valA".to_owned()), map.get_mut("keyA", SystemTime::now()));
    assert_eq!(Some(&"valA".to_owned()), map.get("keyA", SystemTime::now()));
  }

  #[test]
  fn entry_expires_after_time_to_live() {
    let mut map : ExpiringMap<String, String> = get_test_map();

    map.insert("keyA".to_owned(), "valA".to_owned(), SystemTime::now());

    let read_time = SystemTime::now().add(Duration::from_secs(30));
    assert_eq!(Some(&mut "valA".to_owned()), map.get_mut("keyA", read_time));
    assert_eq!(Some(&"valA".to_owned()), map.get("keyA", read_time));

    let read_time_2 = SystemTime::now().add(Duration::from_secs(65));
    assert_eq!(None, map.get_mut("keyA", read_time_2));
    assert_eq!(None, map.get("keyA", read_time_2));
  }

  #[test]
  fn remove_expired_entries() {
    let mut map : ExpiringMap<String, String> = get_test_map();

    map.insert("keyA".to_owned(), "valA".to_owned(), SystemTime::now());

    assert_eq!(1, map.inner.len());

    map.remove_expired_entries(SystemTime::now().add(Duration::from_secs(65)));

    assert_eq!(0, map.inner.len());
  }
}
