#!/usr/bin/env bash
# Compila e instala a Oficina no menu de aplicativos do usuário.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
bin_dir="${XDG_BIN_HOME:-$HOME/.local/bin}"
data_dir="${XDG_DATA_HOME:-$HOME/.local/share}"
apps_dir="$data_dir/applications"
icon_dir="$data_dir/icons/hicolor/scalable/apps"

echo "→ Compilando versão otimizada..."
cargo build --release --manifest-path "$root/Cargo.toml"

echo "→ Instalando binário em $bin_dir/oficina"
install -Dm755 "$root/target/release/oficina" "$bin_dir/oficina"

echo "→ Instalando ícone em $icon_dir/oficina.svg"
install -Dm644 "$root/assets/oficina.svg" "$icon_dir/oficina.svg"

echo "→ Criando atalho em $apps_dir/oficina.desktop"
mkdir -p "$apps_dir"
cat > "$apps_dir/oficina.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=Oficina
GenericName=Gestão de assistência técnica
Comment=Clientes, equipamentos, orçamentos e pagamentos
Exec=$bin_dir/oficina
Icon=oficina
Terminal=false
Categories=Office;Finance;
Keywords=oficina;assistencia;orcamento;clientes;
StartupNotify=true
EOF
chmod 644 "$apps_dir/oficina.desktop"

if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database "$apps_dir" >/dev/null 2>&1 || true
fi

echo
echo "Pronto! A Oficina já aparece no menu de aplicativos."
echo "  Executável: $bin_dir/oficina"
echo "  Atalho:     $apps_dir/oficina.desktop"
