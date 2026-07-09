# Bata Palma e Abra Programas 👏

Um launcher em Python levíssimo e inteligente para Linux. Ele escuta pelo microfone e, ao escutar um estalo/palma (e filtrando vento, sopro ou ruído do ambiente), abre programas que você determinar.

Ideal para atalhos de automação (abrir o terminal, navegador e chat de voz) sem precisar usar o teclado, e sem os falsos positivos irritantes de outras soluções!

## Como funciona?

Para evitar depender de bibliotecas pesadas de machine learning ou de wrappers de áudio como `PyAudio` que costumam falhar ao compilar em distribuições Linux mais novas, este script usa o pacote estável `sounddevice`.

Ao invés de escutar tudo, o script aplica matemática no som em tempo real usando NumPy:
1. **Filtro Passa-Alta (`np.diff`):** Remove frequências lentas instantaneamente (vento, respiração e sopro tornam-se inaudíveis para o sistema).
2. **Janela de Pico e Queda:** Ele checa se o som "subiu de repente" (volume do estalo > 0.08) e se ele "secou" numa fração de segundo (queda drástica < 30%), característica clássica de uma palma.

## Dependências

```bash
# Debian/Ubuntu (ou qualquer distro que o Python 3 reclame do pip)
sudo apt install python3-sounddevice python3-numpy
# Alternativa pip:
pip install sounddevice numpy --break-system-packages
```

## Configuração

Edite o arquivo `clap_launcher.py`:
- `LAUNCH_CMD`: Mude os comandos que você quer que iniciem com a palma (ex: `/usr/bin/firefox`, `flatpak run ...`)
- `THRESHOLD`: O quão "forte" a palma precisa ser (padrão `0.08` para o sinal filtrado).
- `DECAY_RATIO`: A taxa de queda para considerar que o som era agudo/seco (padrão `0.30`).

## Rodando e Testando

Teste se a força das suas palmas bate com a configuração (ele imprimirá se detectou ou não):
```bash
python3 clap_launcher.py --test
```

## Instalação (Serviço Automático no Linux - Systemd)

Para que o programa inicie sozinho no boot rodando levemente em background:

1. Edite o caminho dentro do seu arquivo de serviço (`clap-launcher.service`):
```ini
[Unit]
Description=Bata palma e abra programas
After=network.target

[Service]
ExecStart=/caminho/para/este/repositorio/clap_launcher.py
Restart=always

[Install]
WantedBy=default.target
```

2. Copie para a pasta do systemd do usuário:
```bash
mkdir -p ~/.config/systemd/user/
cp clap-launcher.service ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable --now clap-launcher.service
```