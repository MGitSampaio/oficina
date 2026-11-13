# Oficina — gestão de assistência técnica

Programa desktop em **Rust + egui + SQLite** para uma pequena oficina de conserto de
eletrodomésticos. Controla clientes, equipamentos recebidos, orçamentos e pagamentos,
com interface em português (pt-BR) e dados salvos localmente.

![status](https://img.shields.io/badge/interface-egui-2563EB)
![status](https://img.shields.io/badge/banco-SQLite-15803D)

## Funcionalidades

- **Painel** com indicadores: clientes, equipamentos, valores a receber, recebido no mês.
- **Clientes**: nome, telefone/WhatsApp, e-mail, endereço, observações e contagem de equipamentos.
- **Equipamentos**: tipo, marca, modelo, número de série, cliente responsável e orçamentos vinculados.
- **Orçamentos**: descrição do serviço, mão de obra, peças, desconto, situação
  (`Pendente`, `Aprovado`, `Concluído`, `Recusado`) e saldo em aberto.
- **Pagamentos**: valor, data, forma de pagamento (Pix, dinheiro, cartão etc.) e vínculo com o orçamento.
- **Impressão de orçamento**: abre uma página A4 no navegador já com o diálogo de
  impressão do sistema (também serve para “Salvar em PDF”).
- **Busca e filtros** em todas as listas, com atalhos a partir de cada orçamento.
- **Cadastro rápido** em janelas modais com validação e avisos de erro.
- **Exclusão protegida** por confirmação; exclusões em cascata avisam o que será removido.
- **Dados de exemplo** opcionais para conhecer o sistema.

## Requisitos

- Rust estável (edição 2021) — instale com [rustup](https://rustup.rs/).
- Linux, Windows ou macOS com driver gráfico.
- No Linux são necessárias as bibliotecas de desenvolvimento do X11/Wayland usuais
  (por exemplo `libxcb`, `libxkbcommon`, `libgl1`).
- O SQLite é compilado junto (`rusqlite/bundled`), sem dependência externa.

## Como compilar e executar

```bash
cd oficina

# desenvolvimento
cargo run

# versão otimizada
cargo build --release
./target/release/oficina
```

### Dados de exemplo

```bash
cargo run -- --demo   # cria clientes, equipamentos, orçamentos e pagamentos de exemplo
cargo run             # abre o sistema já com os exemplos
```

Para começar do zero, apague o arquivo do banco (veja abaixo) ou use outra pasta de dados.

## Instalação no menu do Linux

```bash
cargo build --release
install -Dm755 target/release/oficina ~/.local/bin/oficina

mkdir -p ~/.local/share/applications ~/.local/share/icons/hicolor/scalable/apps
install -Dm644 assets/oficina.svg ~/.local/share/icons/hicolor/scalable/apps/oficina.svg
```

Crie `~/.local/share/applications/oficina.desktop`:

```ini
[Desktop Entry]
Type=Application
Name=Oficina
GenericName=Gestão de assistência técnica
Comment=Clientes, equipamentos, orçamentos e pagamentos
Exec=/home/SEU_USUARIO/.local/bin/oficina
Icon=oficina
Terminal=false
Categories=Office;Finance;
Keywords=oficina;assistencia;orcamento;clientes;
```

Depois disso o programa aparece no menu de aplicativos. O `.desktop` pronto
(com o caminho do usuário já preenchido) é gerado por `scripts/install-desktop.sh`.

## Onde ficam os dados

O banco fica em um arquivo SQLite:

- Linux: `~/.local/share/oficina/oficina.db`
- Windows: `%APPDATA%\oficina\oficina.db`
- macOS: `~/Library/Application Support/oficina/oficina.db`

É possível trocar a pasta com a variável `OFICINA_DATA_DIR`:

```bash
OFICINA_DATA_DIR=/tmp/oficina cargo run
```

O banco usa **WAL** e chaves estrangeiras com `ON DELETE CASCADE`, então clientes
removidos levam junto seus equipamentos, orçamentos e pagamentos. Ao excluir um
equipamento, os orçamentos permanecem, mas ficam sem equipamento associado.

## Estrutura do projeto

```
oficina/
├── Cargo.toml
└── src/
    ├── main.rs          # inicialização do eframe e argumentos de linha de comando
    ├── app.rs           # estado da aplicação, menu lateral, modais e avisos
    ├── theme.rs         # paleta de cores e estilo global do egui
    ├── model.rs         # estruturas de domínio (cliente, equipamento, orçamento, pagamento)
    ├── db.rs            # schema SQLite, consultas, dados de exemplo e testes
    ├── util.rs          # formatação de dinheiro e datas (pt-BR)
    └── ui/
        ├── widgets.rs   # cartões, botões, busca, etiquetas e tabelas
        ├── forms.rs     # formulários modais com validação
        ├── dashboard.rs # painel inicial
        ├── clients.rs
        ├── equipment.rs
        ├── quotes.rs
        └── payments.rs
```

## Regras de negócio

- Valores são armazenados em **centavos** (`INTEGER`) para evitar erros de arredondamento.
- O total do orçamento é `mão de obra + peças − desconto`.
- O saldo é `total − soma dos pagamentos`, nunca negativo.
- Datas são gravadas em ISO (`AAAA-MM-DD`) e exibidas em `DD/MM/AAAA`.
- A digitação de valores aceita `120`, `120,50`, `1.234,56` e `R$ 1.234,56`.

## Testes

```bash
cargo test
```

Os testes cobrem conversão de dinheiro/datas, um fluxo CRUD completo no banco
(cliente → equipamento → orçamento → pagamento → exclusões em cascata) e a
renderização sem janela de todas as páginas e formulários em dois tamanhos de tela.

## Atalhos

- `Esc` fecha a janela modal aberta.
- Clique fora de um modal também o fecha.
