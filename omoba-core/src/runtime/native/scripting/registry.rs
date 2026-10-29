//! 映射 `unit_id → UnitScript_TO` 以進行調度。
//! 也保留載入的“RootModule”引用，以便底層 DLL 保留
//! 在整個進程生命週期內都處於活動狀態（H1 - 無熱重載）。

use crate::ability_meta::AbilityDef;
use abi_stable::std_types::{RBox, RErr, ROk, RStr};
use hashbrown::HashMap;
use omb_script_abi::{
    ability::AbilityScript_TO,
    manifest::{Manifest_Ref, RuntimeLuaReloadInfoFFI},
    script::UnitScript_TO,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScriptRuntimeLuaReloadInfo {
    pub module_index: usize,
    pub generation: u64,
    pub hash: String,
}

pub struct ScriptRegistry {
    scripts: HashMap<String, UnitScript_TO<'static, RBox<()>>>,
    /// 保持腳本 DLL `units()` 回傳順序，前端 UI 按鈕依此排序
    order: Vec<String>,
    /// Ability id → (metadata, handler)。Metadata 部分會被複製到 ECS 的
    /// `AbilityRegistry` resource 供 client query；handler 部分供 server
    /// dispatch 技能施放時呼叫。
    abilities: HashMap<String, (AbilityDef, AbilityScript_TO<'static, RBox<()>>)>,
    ability_order: Vec<String>,
    /// 保持清單引用處於活動狀態 → DLL 保持映射狀態。
    _manifests: Vec<Manifest_Ref>,
    /// M3'：每個 manifest 的模組 id 與它註冊的 unit／ability id（與 `_manifests` 同序）。
    modules: Vec<RegisteredModule>,
    /// 熱加載換下的舊版 manifest：DLL 不卸載，引用留著。
    _retired: Vec<Manifest_Ref>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RegisteredModule {
    pub module: String,
    pub units: Vec<String>,
    pub abilities: Vec<String>,
}

impl ScriptRegistry {
    pub fn new() -> Self {
        Self {
            scripts: HashMap::new(),
            order: Vec::new(),
            abilities: HashMap::new(),
            ability_order: Vec::new(),
            _manifests: Vec::new(),
            modules: Vec::new(),
            _retired: Vec::new(),
        }
    }

    /// The registered script modules, in registration order.
    pub fn modules(&self) -> &[RegisteredModule] {
        &self.modules
    }

    pub fn insert_manifest(&mut self, manifest: Manifest_Ref) {
        let mut registered = RegisteredModule {
            module: (manifest.module_id())().as_str().to_owned(),
            ..RegisteredModule::default()
        };
        let units = (manifest.units())();
        for def in units {
            let id: String = def.unit_id.into();
            registered.units.push(id.clone());
            if self.scripts.contains_key(&id) {
                log::warn!("[scripting] duplicate unit_id `{}` — overriding", id);
            } else {
                self.order.push(id.clone());
            }
            self.scripts.insert(id, def.script);
        }

        let abilities = (manifest.abilities())();
        for ability in abilities {
            let def: AbilityDef = match serde_json::from_str(ability.def_json.as_str()) {
                Ok(d) => d,
                Err(e) => {
                    log::error!(
                        "[scripting] failed to parse AbilityDef JSON: {} — payload: {}",
                        e,
                        ability.def_json
                    );
                    continue;
                }
            };
            let id = def.id.clone();
            registered.abilities.push(id.clone());
            if self.abilities.contains_key(&id) {
                log::warn!("[scripting] duplicate ability id `{}` — overriding", id);
            } else {
                self.ability_order.push(id.clone());
            }
            self.abilities.insert(id, (def, ability.script));
        }

        self._manifests.push(manifest);
        self.modules.push(registered);
    }

    /// M3' ② dev hot reload: a new build of one script DLL (same module id) replaces the
    /// scripts and ability metadata it registers; registration order stays. The new build must
    /// register exactly the same unit and ability ids (adding or removing content needs a
    /// restart), else nothing changes. The old DLL stays mapped (abi_stable never unloads); all
    /// state lives in the host, so swapping code loses none.
    pub fn replace_module(&mut self, manifest: Manifest_Ref) -> Result<RegisteredModule, String> {
        let module = (manifest.module_id())().as_str().to_owned();
        let index = self
            .modules
            .iter()
            .position(|registered| registered.module == module)
            .ok_or_else(|| format!("module `{module}` is not loaded; a hot reload only replaces a loaded module"))?;
        let units: Vec<_> = (manifest.units())().into_iter().map(|def| (String::from(def.unit_id), def.script)).collect();
        let mut abilities = Vec::new();
        for ability in (manifest.abilities())() {
            let def: AbilityDef = serde_json::from_str(ability.def_json.as_str())
                .map_err(|error| format!("{module}: ability metadata: {error}"))?;
            abilities.push((def, ability.script));
        }
        let registered = RegisteredModule {
            module: module.clone(),
            units: units.iter().map(|(id, _)| id.clone()).collect(),
            abilities: abilities.iter().map(|(def, _)| def.id.clone()).collect(),
        };
        if registered != self.modules[index] {
            return Err(format!(
                "{module}: the new build registers other units or abilities ({:?} / {:?}, loaded {:?} / {:?}); restart instead",
                registered.units, registered.abilities, self.modules[index].units, self.modules[index].abilities
            ));
        }
        for (id, script) in units {
            self.scripts.insert(id, script);
        }
        for (def, script) in abilities {
            self.abilities.insert(def.id.clone(), (def, script));
        }
        let old = std::mem::replace(&mut self._manifests[index], manifest);
        self._retired.push(old);
        Ok(registered)
    }

    /// Replace the script that runs `ability_id`, keeping its metadata. M3 replays run an
    /// ability once with its behavior IR and once with the Rust handler it replaces.
    /// False when the ability is not registered.
    pub fn replace_ability_script(&mut self, ability_id: &str, script: AbilityScript_TO<'static, RBox<()>>) -> bool {
        match self.abilities.get_mut(ability_id) {
            Some(entry) => { entry.1 = script; true }
            None => false,
        }
    }

    /// Replace the UnitScript of `unit_id` (M3 replays run a hero's IR UnitScript and its Rust one).
    /// False when the unit is not registered.
    pub fn replace_unit_script(&mut self, unit_id: &str, script: UnitScript_TO<'static, RBox<()>>) -> bool {
        match self.scripts.get_mut(unit_id) {
            Some(entry) => { *entry = script; true }
            None => false,
        }
    }

    /// Ability metadata + handler 的 iterate 入口（依 DLL 宣告順序）。
    pub fn iter_abilities(
        &self,
    ) -> impl Iterator<Item = (&str, &AbilityDef, &AbilityScript_TO<'static, RBox<()>>)> {
        self.ability_order.iter().filter_map(|id| {
            self.abilities
                .get(id)
                .map(|(def, script)| (id.as_str(), def, script))
        })
    }

    pub fn ability_count(&self) -> usize {
        self.abilities.len()
    }

    /// 以 ability_id 直查 metadata + handler（供 SkillCast dispatch 使用）。
    pub fn get_ability(
        &self,
        ability_id: &str,
    ) -> Option<&(AbilityDef, AbilityScript_TO<'static, RBox<()>>)> {
        self.abilities.get(ability_id)
    }

    pub fn get(&self, unit_id: &str) -> Option<&UnitScript_TO<'static, RBox<()>>> {
        self.scripts.get(unit_id)
    }

    #[cfg(test)]
    pub(crate) fn insert_ability_for_test(&mut self, def: AbilityDef, script: AbilityScript_TO<'static, RBox<()>>) {
        let id = def.id.clone();
        if !self.abilities.contains_key(&id) {
            self.ability_order.push(id.clone());
        }
        self.abilities.insert(id, (def, script));
    }

    #[cfg(test)]
    pub(crate) fn insert_unit_for_test(
        &mut self,
        unit_id: impl Into<String>,
        script: UnitScript_TO<'static, RBox<()>>,
    ) {
        let unit_id = unit_id.into();
        if !self.scripts.contains_key(&unit_id) {
            self.order.push(unit_id.clone());
        }
        self.scripts.insert(unit_id, script);
    }

    pub fn len(&self) -> usize {
        self.scripts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.scripts.is_empty()
    }

    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.scripts.keys().map(|s| s.as_str())
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &UnitScript_TO<'static, RBox<()>>)> {
        self.scripts.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// 依 DLL `units()` 註冊順序 iterate
    pub fn iter_ordered(&self) -> impl Iterator<Item = (&str, &UnitScript_TO<'static, RBox<()>>)> {
        self.order
            .iter()
            .filter_map(|id| self.scripts.get(id).map(|s| (id.as_str(), s)))
    }

    /// DEV-only runtime Lua content synchronization for loaded script DLLs.
    pub fn reload_runtime_lua_content_dev(
        &self,
        expected_hash: &str,
    ) -> Result<Vec<ScriptRuntimeLuaReloadInfo>, String> {
        let mut reloaded = Vec::new();
        for (module_index, manifest) in self._manifests.iter().enumerate() {
            let result = (manifest.dev_reload_runtime_lua_content())(RStr::from_str(expected_hash));
            match result {
                ROk(RuntimeLuaReloadInfoFFI { generation, hash }) => {
                    let hash = hash.to_string();
                    if hash != expected_hash {
                        return Err(format!(
                            "script module #{} runtime Lua content hash mismatch: expected {}, got {}",
                            module_index, expected_hash, hash
                        ));
                    }
                    reloaded.push(ScriptRuntimeLuaReloadInfo {
                        module_index,
                        generation,
                        hash,
                    });
                }
                RErr(err) => {
                    return Err(format!(
                        "script module #{} runtime Lua content reload failed: {}",
                        module_index, err
                    ));
                }
            }
        }
        Ok(reloaded)
    }
}

impl Default for ScriptRegistry {
    fn default() -> Self {
        Self::new()
    }
}
