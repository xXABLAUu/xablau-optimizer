// Embute o manifesto (requireAdministrator) e o ícone do kit de emergência.
// Pula com aviso quando as ferramentas de recurso não estão disponíveis (ou XABLAU_SKIP_RC=1).
fn main() {
    println!("cargo:rerun-if-changed=recursos/restaurar.rc");
    println!("cargo:rerun-if-env-changed=XABLAU_SKIP_RC");
    let alvo_windows = std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows");
    if !alvo_windows || std::env::var("XABLAU_SKIP_RC").is_ok() {
        return;
    }
    if let Err(e) = embed_resource::compile("recursos/restaurar.rc", embed_resource::NONE).manifest_optional() {
        println!("cargo:warning=Ícone/manifesto do kit não foram embutidos: {e}");
    }
}
