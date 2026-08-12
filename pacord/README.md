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
