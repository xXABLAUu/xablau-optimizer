// Embute o manifesto (asInvoker) e o ícone da bandeja no executável do vigia.
//
// No Windows com as ferramentas do Rust (MSVC) isso funciona direto. Em verificações feitas
// em outro sistema operacional, ou quando faltam as ferramentas de recurso, o embed é pulado
// com um aviso em vez de quebrar a compilação (XABLAU_SKIP_RC=1 força pular).
fn main() {
    println!("cargo:rerun-if-changed=recursos/vigia.rc");
    println!("cargo:rerun-if-env-changed=XABLAU_SKIP_RC");
    let alvo_windows = std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows");
    let pular = std::env::var("XABLAU_SKIP_RC").is_ok();
    if !alvo_windows || pular {
        return;
    }
    if let Err(e) = embed_resource::compile("recursos/vigia.rc", embed_resource::NONE).manifest_optional() {
        println!("cargo:warning=Ícone/manifesto do vigia não foram embutidos: {e}");
    }
}
