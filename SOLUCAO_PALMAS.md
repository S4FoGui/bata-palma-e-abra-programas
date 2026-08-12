# Solução Profissional com Biblioteca Especializada (clap-detector)

Para garantir o melhor desempenho e precisão, o projeto foi migrado para utilizar a biblioteca especializada **`clap-detector`**. Esta biblioteca utiliza filtros de banda (Bandpass) e algoritmos de processamento de sinal mais robustos que os métodos manuais, resultando em uma detecção muito mais confiável.

---

## 1. O que mudou?

| Característica | Detalhe Técnico |
| :--- | :--- |
| **Biblioteca Core** | Migrado de `sounddevice` puro para `clap-detector` (que usa `PyAudio` e `SciPy`). |
| **Filtro de Frequência** | Aplica um filtro que foca apenas na faixa de frequência das palmas (200Hz a 3200Hz), ignorando ruídos graves como vento ou motores. |
| **Detecção de Padrões** | A biblioteca é capaz de diferenciar uma palma simples de palmas duplas, o que permite expansões futuras no seu script. |

---

## 2. Instalação das Novas Dependências

Como agora usamos bibliotecas de processamento de áudio mais avançadas, você precisará instalar algumas dependências do sistema no seu Linux:

```bash
# 1. Instalar dependências de áudio do sistema (Debian/Ubuntu)
sudo apt update
sudo apt install -y portaudio19-dev python3-pyaudio build-essential python3-dev

# 2. Instalar as bibliotecas Python
pip install clap-detector scipy --break-system-packages
```

---

## 3. Como usar a nova versão

### Passo 1: Testar a sensibilidade
Execute o script no modo de teste para ver como ele se comporta no seu ambiente:

```bash
python3 clap_launcher.py --test
```

*   O terminal mostrará uma lista de dispositivos de áudio. O script usará o padrão do sistema automaticamente.
*   Bata palmas e veja a mensagem `>>> [PALMA DETECTADA!] <<<` aparecer.

### Passo 2: Ajustar a precisão (Opcional)
Dentro do arquivo `clap_launcher.py`, você pode ajustar estas variáveis se a detecção estiver muito sensível ou pouco sensível:
- `threshold_bias`: Aumente este valor (ex: para 8000) se ele estiver pegando muito ruído. Diminua (ex: para 4000) se ele não estiver pegando suas palmas.
- `lowcut` e `highcut`: Definem a faixa de frequência. O padrão (200-3200) já é excelente para a maioria dos ambientes.

### Passo 3: Rodar em produção
Para deixar o programa rodando e abrindo seus aplicativos:

```bash
python3 clap_launcher.py
```

---

## 4. Vantagem de Desempenho
Ao usar a biblioteca `clap-detector`, o processamento de sinal é feito através do **SciPy**, que é altamente otimizado para operações matemáticas complexas em áudio. Isso garante que o programa consuma o mínimo de CPU possível enquanto mantém uma vigilância constante.
