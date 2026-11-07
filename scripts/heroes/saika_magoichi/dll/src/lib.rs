//! hero_b01.dll：只匯出 hero_b01_impl 的 manifest。技能本體在上一層的 rlib，base_content 的測試連結那個
//! rlib；這個 cdylib 沒有人依賴，所以測試建置不會覆寫 DLL。
#![deny(warnings)]

#[abi_stable::export_root_module]
fn get_manifest() -> hero_b01_impl::Manifest_Ref {
    hero_b01_impl::get_manifest()
}
