#[macro_export]
macro_rules! id_realm {
    (
        id_generation:   via_max_value_in_domain
        uint:            ($itype:ty) $write_itype:ident 
        ID:              ($(#[$ID_attrs:meta])* $($pub_ID:ident)*) $ID:ident
        IDDomain:        ($(#[$IDDomain_attrs:meta])* $($pub_IDDomain:ident)*) $IDDomain:ident
        IDHasher:        ($(#[$IDHasher_attrs:meta])* $($pub_IDHasher:ident)*) $IDHasher:ident
        IDHasherBuilder: ($(#[$IDHasherBuilder_attrs:meta])* $($pub_IDHasherBuilder:ident)*) $IDHasherBuilder:ident
        IDMap:           ($(#[$IDMap_attrs:meta])* $($pub_IDMap:ident)*) $IDMap:ident
        IDRealm:         ($(#[$IDRealm_attrs:meta])* $($pub_IDRealm:ident)*) $IDRealm:ident
    ) => {
        /// A strongly-type ID value.
        $(#[$ID_attrs])*
        #[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
        $($pub_ID)* struct $ID($itype);

        /// A monotonically-increasing ID generator.
        $(#[$IDDomain_attrs])*
        #[derive(Debug, Clone, Hash, PartialEq, Eq)]
        $($pub_IDDomain)* struct $IDDomain {
            current_highest: $itype,
        }

        $(#[$IDHasher_attrs])*
        #[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
        $($pub_IDHasher)* struct $IDHasher($itype);

        $(#[$IDHasherBuilder_attrs])*
        #[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
        $($pub_IDHasherBuilder)* struct $IDHasherBuilder;

        $(#[$IDMap_attrs])*
        $($pub_IDMap)* type $IDMap<T> = ::std::collections::HashMap<$ID, T, $IDHasherBuilder>;

        $(#[$IDRealm_attrs])*
        #[derive(Debug, PartialEq, Eq)]
        $($pub_IDRealm)* struct $IDRealm<T> {
            domain: $IDDomain,
            map: $IDMap<T>,
        }

        impl ::std::hash::Hasher for $IDHasher {
            fn finish(&self) -> u64 {
                self.0 as _
            }
            fn write(&mut self, _bytes: &[u8]) { unreachable!{} }
            fn $write_itype(&mut self, i: $itype) { self.0 = i; }
        }

        impl ::std::hash::BuildHasher for $IDHasherBuilder {
            type Hasher = $IDHasher;
            fn build_hasher(&self) -> Self::Hasher {
                Default::default()
            }
        }

        impl $ID {
            pub fn from_raw(value: $itype) -> Self {
                $ID(value)
            }
            pub fn get_raw(&self) -> $itype {
                self.0
            }
        }

        impl $IDDomain {
            pub fn new_empty() -> Self {
                Self {
                    current_highest: 0,
                }
            }
            pub fn from_ids<I>(ids: I) -> Self 
                where I: IntoIterator<Item=$ID>
            {
                let mut slf = Self::new_empty();
                slf.include_ids(ids);
                slf
            }
            pub fn include_ids<I>(&mut self, ids: I) 
                where I: IntoIterator<Item=$ID>
            {
                for id in ids {
                    self.include_id(id);
                }
            }
            pub fn include_id(&mut self, id: $ID) {
                self.current_highest = ::std::cmp::max(self.current_highest, id.get_raw());
            }
            pub fn generate_new_id(&mut self) -> $ID {
                self.current_highest += 1;
                $ID(self.current_highest)
            }
        }
        impl<T> $IDRealm<T> {
            pub fn from_iterator<I>(iterator: I) -> Self where I: IntoIterator<Item=($ID, T)> {
                let mut slf = Self::new_empty();
                for (id, value) in iterator {
                    slf.insert_missing(id, value);
                }
                slf
            }
            pub fn new_empty() -> Self {
                Self::with_capacity(0)
            }
            pub fn with_capacity(capacity: usize) -> Self {
                Self {
                    domain: $IDDomain::new_empty(),
                    map: $IDMap::with_capacity_and_hasher(capacity, Default::default()),
                }
            }
            pub fn insert_or_replace(&mut self, id: $ID, value: T) -> Option<T> {
                self.domain.include_id(id);
                self.map.insert(id, value)
            }
            pub fn insert_missing(&mut self, id: $ID, value: T) {
                self.domain.include_id(id);
                let old = self.insert_or_replace(id, value);
                assert!(old.is_none());
            }
            pub fn replace_existing(&mut self, id: $ID, value: T) -> T {
                self.insert_or_replace(id, value).unwrap()
            }
            pub fn insert_new_and_get_id(&mut self, value: T) -> $ID {
                let id = self.generate_new_id();
                self.insert_missing(id, value);
                id
            }
            pub fn generate_new_id(&mut self) -> $ID {
                self.domain.generate_new_id()
            }
            pub fn ids(&self) -> ::std::collections::hash_map::Keys<$ID, T> { self.map.keys() }
            pub fn values(&self) -> ::std::collections::hash_map::Values<$ID, T> { self.map.values() }
            pub fn values_mut(&mut self) -> ::std::collections::hash_map::ValuesMut<$ID, T> { self.map.values_mut() }
            pub fn iter(&self) -> ::std::collections::hash_map::Iter<$ID, T> { self.map.iter() }
            pub fn iter_mut(&mut self) -> ::std::collections::hash_map::IterMut<$ID, T> { self.map.iter_mut() }
            pub fn entry(&mut self, id: $ID) -> ::std::collections::hash_map::Entry<$ID, T> { self.map.entry(id) }
            pub fn len(&self) -> usize { self.map.len() }
            pub fn is_empty(&self) -> bool { self.map.is_empty() }
            pub fn drain(&mut self) -> ::std::collections::hash_map::Drain<$ID, T> { self.map.drain() }
            pub fn clear(&mut self) { self.map.clear(); self.domain = $IDDomain::new_empty(); }
            pub fn get(&self, id: $ID) -> Option<&T> { self.map.get(&id) }
            pub fn get_mut(&mut self, id: $ID) -> Option<&mut T> { self.map.get_mut(&id) }
            pub fn contains_id(&self, id: $ID) -> bool { self.map.contains_key(&id) }
            pub fn remove(&mut self, id: $ID) -> Option<T> { self.map.remove(&id) }
        }

        impl<T> ::std::ops::Index<$ID> for $IDRealm<T> {
            type Output = T;
            fn index(&self, id: $ID) -> &T {
                &self.map[&id]
            }
        }
        impl<T> ::std::ops::IndexMut<$ID> for $IDRealm<T> {
            fn index_mut(&mut self, id: $ID) -> &mut T {
                self.get_mut(id).unwrap()
            }
        }

        impl<T> IntoIterator for $IDRealm<T> {
            type Item = ($ID, T);
            type IntoIter = ::std::collections::hash_map::IntoIter<$ID, T>;
            fn into_iter(self) -> Self::IntoIter {
                self.map.into_iter()
            }
        }

        impl<'a, T> IntoIterator for &'a $IDRealm<T> {
            type Item = (&'a $ID, &'a T);
            type IntoIter = ::std::collections::hash_map::Iter<'a, $ID, T>;
            fn into_iter(self) -> Self::IntoIter {
                self.map.iter()
            }
        }

        impl<'a, T> IntoIterator for &'a mut $IDRealm<T> {
            type Item = (&'a $ID, &'a mut T);
            type IntoIter = ::std::collections::hash_map::IterMut<'a, $ID, T>;
            fn into_iter(self) -> Self::IntoIter {
                self.map.iter_mut()
            }
        }
    }
}

#[allow(dead_code)]
mod it_works {
    id_realm!{
        id_generation:   via_max_value_in_domain
        uint:            (u32) write_u32
        ID:              (pub) FoobarID
        IDDomain:        (pub) FoobarIDDomain
        IDHasher:        (pub) FoobarIDHasher
        IDHasherBuilder: (pub) FoobarIDHasherBuilder
        IDMap:           (pub) FoobarIDMap
        IDRealm:         (pub) FoobarIDRealm
    }
    id_realm!{
        id_generation:   via_max_value_in_domain
        uint:            (u64) write_u64
        ID:              () XyzzyID
        IDDomain:        () XyzzyIDDomain
        IDHasher:        () XyzzyIDHasher
        IDHasherBuilder: () XyzzyIDHasherBuilder
        IDMap:           () XyzzyIDMap
        IDRealm:         () XyzzyIDRealm
    }
}

