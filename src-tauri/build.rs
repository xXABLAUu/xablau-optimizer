// Build do app Tauri: gera o contexto, embute o manifesto (requireAdministrator) e confere, em
// modo release, que os binários auxiliares (vigia, kit, PresentMon) estão presentes.
fn main() {
    tauri_build::build();
}
