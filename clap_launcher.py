#!/usr/bin/env python3
"""
Clap Launcher - Detecta palmas (pico + queda rápida + altas frequências)
Usa filtro diferencial passa-alta no NumPy para barrar vento e sopros.
"""
import sys
import time
import subprocess
import numpy as np
import sounddevice as sd

# Configuração (Ajustado para o novo cálculo focado em som "agudo" da palma)
THRESHOLD = 0.08      # Volume mínimo do estalo (como tiramos o grave do vento, o valor cai)
DECAY_RATIO = 0.30    # A queda após o estalo tem que ser drástica
COOLDOWN = 2.0        # Segundos entre uma detecção e outra

LAUNCH_CMD = (
    "x-terminal-emulator -e hermes & "
    "flatpak run com.brave.Browser & "
    "flatpak run com.discordapp.Discord"
)

def run_test():
    print("--- MODO DE TESTE ANTI-VENTO ---")
    print("Sopre no microfone. O volume deve ficar baixo.")
    print("Bata palmas. O volume deve dar um salto e marcar [PALMA DETECTADA!].")
    print("Ctrl+C para sair.\n")
    
    last_trigger = 0
    history = [0.0, 0.0, 0.0, 0.0]
    
    def callback(indata, frames, time_info, status):
        nonlocal last_trigger, history
        # O segredo: np.diff tira a lentidão da onda (som grave/vento) e expõe a velocidade (estalo)
        high_freq_data = np.diff(indata, axis=0)
        rms = float(np.sqrt(np.mean(high_freq_data**2)))
        history.append(rms)
        history.pop(0)
        
        pico = history[2]
        atual = history[3]
        
        if pico > 0.015:
            print(f"Som -> Volume Filt. : {pico:.3f} | Queda: {(atual/pico if pico > 0 else 1):.2f}")
            
        if pico > THRESHOLD and (atual / pico) < DECAY_RATIO:
            now = time.time()
            if (now - last_trigger) > COOLDOWN:
                last_trigger = now
                print("\n>>> [PALMA DETECTADA!] <<<\n")

    with sd.InputStream(callback=callback, samplerate=44100, blocksize=2048, channels=1):
        while True:
            time.sleep(0.5)

def run_daemon():
    print(f"Iniciando serviço. THRESHOLD: {THRESHOLD}")
    last_trigger = 0
    history = [0.0, 0.0, 0.0, 0.0]
    
    def callback(indata, frames, time_info, status):
        nonlocal last_trigger, history
        high_freq_data = np.diff(indata, axis=0)
        rms = float(np.sqrt(np.mean(high_freq_data**2)))
        history.append(rms)
        history.pop(0)
        
        pico = history[2]
        atual = history[3]
        
        if pico > THRESHOLD and (atual / pico) < DECAY_RATIO:
            now = time.time()
            if (now - last_trigger) > COOLDOWN:
                last_trigger = now
                print(f"[CLAP] Estalo Forte detectado ({pico:.3f}). Abrindo apps...")
                subprocess.Popen(LAUNCH_CMD, shell=True, start_new_session=True)

    with sd.InputStream(callback=callback, samplerate=44100, blocksize=2048, channels=1):
        while True:
            time.sleep(1)

if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] == "--test":
        try:
            run_test()
        except KeyboardInterrupt:
            print("\nSaindo.")
    else:
        run_daemon()