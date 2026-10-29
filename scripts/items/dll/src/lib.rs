//! items.dll：只匯出 items_impl 的 manifest。道具本體在上一層的 rlib；這個 cdylib 沒有人依賴，
//! 所以測試建置不會覆寫 DLL。
#![deny(warnings)]

#[abi_stable::export_root_module]
fn get_manifest() -> items_impl::Manifest_Ref {
    items_impl::get_manifest()
}
