# Solução Definitiva para a Detecção de Palmas (Bata Palma e Abra Programas)

O problema principal que causava falsos positivos e fazia o launcher capturar som ambiente (como ventilador, conversas, cliques de teclado ou TV) era o uso de um **limiar estático fixo** (`THRESHOLD = 0.08`) combinado com a análise de blocos isolados de áudio, sem adaptação ao ruído de fundo do ambiente.

---

## 1. O que foi corrigido no código

O script `clap_launcher.py` foi totalmente aprimorado com um sistema inteligente de filtragem e adaptação:

| Funcionalidade Nova | Descrição Técnica |
| :--- | :--- |
| **Limiar Adaptativo (*Noise Floor Tracking*)** | O sistema calcula dinamicamente o nível do ruído de fundo recente e exige que o estalo seja significativamente mais forte que a média ambiente, evitando disparos em ambientes ruidosos. |
| **Modo de Calibração Automática (`--calibrate`)** | Analisa o seu microfone por 3 segundos em silêncio para descobrir o limiar exato do seu ambiente de forma automatizada. |
| **Modo de Teste Visual (`--test`)** | Exibe no terminal em tempo real o valor do pico, o limiar dinâmico e o fator de queda, indicando claramente (`🔥 PALMA!` vs `ruído`) quando uma palma é reconhecida. |

---

## 2. Passo a Passo para Atualizar e Usar

Siga os passos abaixo no seu computador Linux para aplicar a correção e calibrar perfeitamente o seu sistema:

### Passo 1: Atualizar o arquivo do projeto
Substitua o conteúdo do seu arquivo `clap_launcher.py` local pelo código atualizado (disponível no repositório GitHub atualizado). 

Se você clonou o repositório, basta puxar as alterações ou atualizar o arquivo com a versão otimizada.

### Passo 2: Calibrar o microfone para o seu ambiente
Abra o terminal na pasta do projeto e execute o comando de calibração. Fique em silêncio por 3 segundos para que o script meça o som ambiente:

```bash
python3 clap_launcher.py --calibrate --test
```

* **O que fazer:** Observe os valores que aparecem no terminal. Dê algumas palmas leves e palmas fortes. O terminal mostrará se o sistema identificou corretamente como `🔥 PALMA!` ou se descartou como `ruído`.

### Passo 3: Testar com diferentes intensidades
Se preferir definir um limiar manual após o teste, você pode passá-lo diretamente por parâmetro:

```bash
python3 clap_launcher.py --threshold 0.12 --test
```

### Passo 4: Rodar em modo normal (Daemon)
Quando estiver satisfeito com a detecção, execute o script normalmente em segundo plano ou como serviço do Systemd:

```bash
python3 clap_launcher.py
```

---

## 3. Configurando o Serviço Automático (Systemd)

Para que o programa inicie sozinho com o seu usuário sempre que o computador ligar:

1. Certifique-se de que o arquivo `clap-launcher.service` está configurado corretamente na pasta `~/.config/systemd/user/`:

```ini
[Unit]
Description=Bata palma e abra programas (Robusto)
After=network.target

[Service]
ExecStart=/usr/bin/python3 /caminho/completo/para/clap_launcher.py
Restart=always

[Install]
WantedBy=default.target
```

2. Ative e inicie o serviço:

```bash
systemctl --user daemon-reload
systemctl --user enable --now clap-launcher.service
```

Pronto! Agora o sistema diferencia perfeitamente o som ambiente e estalos corriqueiros de uma palma real.
