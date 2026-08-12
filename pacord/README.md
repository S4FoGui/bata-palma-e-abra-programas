# PACORD

PACORD é um protótipo de colaboração remota consentida para Linux, escrito em Rust, com interface em preto e branco. O produto final foi planejado para usar uma rede ZeroTier já configurada, permitir até oito participantes, exibir cursores identificados por apelido e indicar controles ativos.

## Estado atual

Esta entrega implementa o **Módulo 1: dispositivos virtuais via uinput** e uma interface inicial em `egui`/`eframe`. O módulo cria, por cliente aprovado pelo host, dispositivos virtuais separados para teclado, mouse e controle. Também existe um perfil TOML para sensibilidade do mouse, curva de aceleração, zona morta, inversão do eixo Y e remapeamento de botões.

A tela atual é uma base visual e de gerenciamento: ela mostra a lista de participantes, permissões independentes de mouse/teclado/controle, o limite de oito participantes e a área de sobreposição com apelido e marcador `[PAD]`. A captura real de tela, transporte ZeroTier/TCP, captura local de eventos, autorização criptográfica e sessões isoladas com Cage/Xephyr ficam para os módulos seguintes; portanto, **não trate esta versão como um produto de acesso remoto pronto para produção**.

## Requisitos de segurança e consentimento

O host deve aprovar explicitamente cada participante e cada classe de entrada. O PACORD não deve iniciar uma sessão oculta, contornar autenticação, instalar persistência ou aceitar comandos fora de uma sessão aprovada. A futura camada de rede deve autenticar o host e o cliente, limitar a oito conexões, expirar permissões e exibir um indicador visível enquanto a entrada remota estiver habilitada.

O acesso a `/dev/uinput` é concedido por uma regra local do sistema. O PACORD não deve ser executado permanentemente como root.

## Dependências de compilação

### Debian/Ubuntu

```bash
sudo apt update
sudo apt install -y build-essential pkg-config libudev-dev libasound2-dev \
  libx11-dev libxi-dev libgl1-mesa-dev
```

### Arch Linux

```bash
sudo pacman -S --needed base-devel pkgconf systemd-libs alsa-lib \
  libx11 libxi mesa
```

É necessário ter Rust estável e Cargo disponíveis. Depois, compile com:

```bash
cargo check
cargo test
cargo run
```

## Configuração de uinput sem root permanente

Crie a regra abaixo como `/etc/udev/rules.d/99-pacord-uinput.rules`:

```udev
KERNEL=="uinput", GROUP="input", MODE="0660", OPTIONS+="static_node=uinput"
```

A forma recomendada é salvar o arquivo com o nome correto e então adicionar o usuário ao grupo `input`:

```bash
sudo install -m 0644 packaging/99-pacord-uinput.rules /etc/udev/rules.d/99-pacord-uinput.rules
sudo usermod -aG input "$USER"
sudo udevadm control --reload-rules
sudo udevadm trigger --subsystem-match=misc
```

Saia e entre novamente na sessão gráfica para que a associação de grupo seja recarregada. Confirme o dispositivo antes de executar o PACORD:

```bash
id -nG | tr ' ' '\n' | grep '^input$'
stat -c '%A %G %n' /dev/uinput
```

Se `/dev/uinput` não existir, carregue o módulo apenas como operação administrativa de configuração:

```bash
sudo modprobe uinput
```

O arquivo de regra só controla permissões do dispositivo local. Ele não abre portas de rede e não configura o ZeroTier.

## Perfil de personalização

Na primeira execução, o perfil `pacord_profile.toml` é criado no diretório de trabalho. Um exemplo equivalente ao perfil padrão é:

```toml
nickname = "PACORD_User"

[mouse]
sensitivity = 1.0
acceleration_curve = 1.0

[controller]
deadzone = 0.15
invert_y = false

[controller.button_map]
A = 304
B = 305
X = 307
Y = 308
```

A aplicação não deve aceitar valores de sensibilidade ou zona morta fora de limites razoáveis na camada de rede. O módulo de configuração é local e deverá ser validado novamente quando a personalização for conectada aos eventos recebidos.

## Arquitetura prevista

| Módulo | Estado | Responsabilidade |
|---|---|---|
| Dispositivos virtuais | Implementado inicialmente | Criar e emitir eventos em teclado, mouse e controle via uinput. |
| Captura local | Planejado | Ler teclado/mouse por evdev e controle por gilrs/SDL2. |
| Host e permissões | Interface inicial | Manter participantes, permissões e limite de oito conexões. |
| Rede | Planejado | Usar IP da rede ZeroTier; a autenticação e o enquadramento de mensagens ainda precisam ser implementados. |
| Captura de tela | Planejado | PipeWire/xdg-desktop-portal em Wayland e XShm como fallback em X11. |
| Sessão isolada | Planejado | Solicitar aprovação antes de iniciar Cage ou Xephyr. |
| Overlay | Demonstração visual | Cursores identificados e marcador de controle na área de visualização. |

## Limitações importantes desta entrega

A versão atual não injeta eventos no desktop até que `/dev/uinput` esteja disponível, não captura a tela, não transmite vídeo e não conecta clientes pela rede. A lista de participantes e os cursores mostrados na interface são dados de demonstração. O próximo incremento deve substituir os dados simulados por um protocolo autenticado, com autorização explícita e testes de integração em X11 e Wayland.

## Licença e desenvolvimento

O repositório original contém outros arquivos não relacionados ao PACORD. O projeto do PACORD está isolado em `pacord/` para que sua evolução possa ser feita com commits pequenos e reversíveis.

## Módulo 2 — captura e transporte

O Módulo 2 adiciona dois backends de captura e um transporte de frames:

| Backend | Sessão | Implementação |
|---|---|---|
| `WaylandPipeWire` | KDE Plasma/Wayland | `ashpd` solicita `CreateSession`, `SelectSources` e `Start` ao portal; o nó autorizado é conectado ao PipeWire por `pipewire-rs`. |
| `X11XShm` | X11 | `x11-dl` verifica MIT-SHM e usa `XShmCreateImage`/`XShmGetImage` para copiar o root window. A ausência da extensão produz erro explícito. |
| Transporte | ZeroTier | `transport.rs` escuta no `SocketAddr` fornecido, executa desafio HMAC-SHA-256, enquadra mensagens com tamanho limitado e distribui JPEGs para no máximo oito clientes. |

O primeiro fluxo validável é composto pelo host executando uma captura autorizada, convertendo cada frame para JPEG e publicando os frames em um broadcast limitado; o cliente envia apelido e prova de posse do segredo compartilhado e recebe apenas frames depois de ser aceito. O segredo não é enviado pela rede. O endereço de bind deve ser o IPv4 da interface ZeroTier, e não `0.0.0.0`.

### Dependências adicionais

No Debian/Ubuntu, além das dependências do Módulo 1, instale:

```bash
sudo apt install -y libpipewire-0.3-dev libspa-0.2-dev libxext-dev \
  libxdamage-dev libclang-dev clang
```

No Arch Linux, instale os equivalentes:

```bash
sudo pacman -S --needed pipewire libpipewire clang libxext libxdamage
```

No Wayland, o fluxo depende de `xdg-desktop-portal`, do backend KDE correspondente e de PipeWire em execução na sessão do usuário. O diálogo de seleção de tela/janela pertence ao portal e precisa ser confirmado localmente pelo host; o PACORD não deve tentar contorná-lo.

### Limitações conscientes

A captura PipeWire aceita inicialmente os formatos brutos RGBA, RGBx, BGRx e RGB; outros formatos negociados são descartados pelo worker até que exista um conversor explícito. A codificação JPEG é uma etapa de validação de latência e largura de banda, não o codec final de produção. A janela interativa completa ainda depende da integração do receptor com a textura da interface e do Módulo 3 de entrada remota; esta etapa transmite vídeo e não autoriza controle por si só.

O protocolo usa autenticação de aplicação além da rede ZeroTier. A rede overlay fornece o caminho IP, mas não concede automaticamente uma sessão PACORD. O host precisa controlar o segredo, aprovar cada cliente e encerrar o processo para interromper todas as transmissões.

### Executar o fluxo host–cliente

Gere um segredo compartilhado fora do processo e entregue-o aos dois participantes por um canal confiável. Não use o valor de exemplo em produção:

```bash
export PACORD_SECRET='substitua-por-um-segredo-com-mais-de-16-bytes'
```

No host X11, usando o IPv4 atribuído pela interface ZeroTier:

```bash
cargo run --release --bin pacord-host -- x11 10.147.20.5:7777
```

No host KDE Wayland, o portal abrirá o diálogo de seleção local:

```bash
cargo run --release --bin pacord-host -- wayland 10.147.20.5:7777
```

Na máquina cliente autorizada, execute a janela visualizadora apontando para o endereço ZeroTier do host:

```bash
cargo run --release --bin pacord-viewer -- 10.147.20.5:7777 Alice
```

O viewer recebe apenas frames após concluir o desafio HMAC-SHA-256. Ele mantém a imagem mais recente em uma janela `egui`; ainda não envia teclado, mouse ou controle, pois essa autorização pertence ao módulo de entrada posterior.

## Referências técnicas

[1] [XDG Desktop Portal — ScreenCast](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.ScreenCast.html)

[2] [PipeWire — Tutorial: Capturing Video Frames](https://docs.pipewire.org/page_tutorial5.html)

[3] [ashpd — módulo ScreenCast em Rust](https://bilelmoussaoui.github.io/ashpd/ashpd/desktop/screencast/index.html)

[4] [PipeWire Rust bindings](https://pipewire.pages.freedesktop.org/pipewire-rs/pipewire/)

[5] [ZeroTier — Client Configuration](https://docs.zerotier.com/config/)

[6] [X.Org — MIT Shared Memory Extension](https://xorg.freedesktop.org/releases/X11R7.7/doc/xextproto/shm.html)
