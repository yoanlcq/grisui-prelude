id_realm!{
    id_generation:   via_max_value_in_domain
    uint:            (u32) write_u32
    ID:              (#[derive(Serialize, Deserialize)] pub) TagID
    IDDomain:        (#[derive(Serialize, Deserialize)] pub) TagIDDomain
    IDHasher:        (pub) TagIDHasher
    IDHasherBuilder: (pub) TagIDHasherBuilder
    IDMap:           (pub) TagIDMap
    IDRealm:         (pub) TagIDRealm
}
id_realm!{
    id_generation:   via_max_value_in_domain
    uint:            (u32) write_u32
    ID:              (#[derive(Serialize, Deserialize)] pub) PaletteEntryID
    IDDomain:        (#[derive(Serialize, Deserialize)] pub) PaletteEntryIDDomain
    IDHasher:        (pub) PaletteEntryIDHasher
    IDHasherBuilder: (pub) PaletteEntryIDHasherBuilder
    IDMap:           (pub) PaletteEntryIDMap
    IDRealm:         (pub) PaletteEntryIDRealm
}
id_realm!{
    id_generation:   via_max_value_in_domain
    uint:            (u32) write_u32
    ID:              (#[derive(Serialize, Deserialize)] pub) MeshID
    IDDomain:        (#[derive(Serialize, Deserialize)] pub) MeshIDDomain
    IDHasher:        (pub) MeshIDHasher
    IDHasherBuilder: (pub) MeshIDHasherBuilder
    IDMap:           (pub) MeshIDMap
    IDRealm:         (pub) MeshIDRealm
}
id_realm!{
    id_generation:   via_max_value_in_domain
    uint:            (u32) write_u32
    ID:              (#[derive(Serialize, Deserialize)] pub) SceneID
    IDDomain:        (#[derive(Serialize, Deserialize)] pub) SceneIDDomain
    IDHasher:        (pub) SceneIDHasher
    IDHasherBuilder: (pub) SceneIDHasherBuilder
    IDMap:           (pub) SceneIDMap
    IDRealm:         (pub) SceneIDRealm
}
id_realm!{
    id_generation:   via_max_value_in_domain
    uint:            (u32) write_u32
    ID:              (#[derive(Serialize, Deserialize)] pub) SaveID
    IDDomain:        (#[derive(Serialize, Deserialize)] pub) SaveIDDomain
    IDHasher:        (pub) SaveIDHasher
    IDHasherBuilder: (pub) SaveIDHasherBuilder
    IDMap:           (pub) SaveIDMap
    IDRealm:         (pub) SaveIDRealm
}
id_realm!{
    id_generation:   via_max_value_in_domain
    uint:            (u32) write_u32
    ID:              (#[derive(Serialize, Deserialize)] pub) EntityID
    IDDomain:        (#[derive(Serialize, Deserialize)] pub) EntityIDDomain
    IDHasher:        (pub) EntityIDHasher
    IDHasherBuilder: (pub) EntityIDHasherBuilder
    IDMap:           (pub) EntityIDMap
    IDRealm:         (pub) EntityIDRealm
}

