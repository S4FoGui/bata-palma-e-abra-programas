#!/usr/bin/env python3
"""
Clap Launcher - Detecta palmas com adaptação de ruído ambiente e filtro diferencial.
Melhorias:
- Limiar adaptativo (Noise Floor Tracking) para lidar com ambientes barulhentos.
- Modo de calibração automática (--calibrate).
- Modo de teste refinado (--test).
"""
import sys
import time
import subprocess
import numpy as np
import sounddevice as sd

# Configurações padrão
DEFAULT_THRESHOLD_FACTOR = 3.5  # Quantas vezes acima do ruído de fundo o pico deve estar
DECAY_RATIO = 0.35              # Taxa de queda após o estalo
COOLDOWN = 2.0                  # Segundos entre uma detecção e outra

LAUNCH_CMD = (
    "x-terminal-emulator -e hermes & "
    "flatpak run com.brave.Browser & "
    "flatpak run com.discordapp.Discord"
)

def calibrate_mic(duration=3.0, samplerate=44100, blocksize=2048):
    print(f"[Calibração] Silêncio por favor... Medindo ruído ambiente por {duration} segundos.")
    noise_levels = []
    
    def callback(indata, frames, time_info, status):
        high_freq_data = np.diff(indata, axis=0)
        rms = float(np.sqrt(np.mean(high_freq_data**2)))
        noise_levels.append(rms)

    with sd.InputStream(callback=callback, samplerate=samplerate, blocksize=blocksize, channels=1):
        time.sleep(duration)
        
    if not noise_levels:
        return 0.02
        
    avg_noise = np.mean(noise_levels)
    max_noise = np.max(noise_levels)
    recommended_threshold = max(0.015, max_noise * DEFAULT_THRESHOLD_FACTOR)
    print(f"[Calibração] Ruído médio: {avg_noise:.4f} | Pico de ruído: {max_noise:.4f}")
    print(f"[Calibração] Limiar recomendado (Threshold): {recommended_threshold:.4f}\n")
    return recommended_threshold

def run_test(threshold):
    print("--- MODO DE TESTE ROBUSTO ---")
    print(f"Limiar atual (Threshold): {threshold:.4f}")
    print("Faça barulhos normais (falar, digitar, ventilador) e depois bata palmas.")
    print("Ctrl+C para sair.\n")
    
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
        
        # Atualiza o piso de ruído dinamicamente no histórico recente
        noise_floor = np.mean(history[:2])
        dynamic_thresh = max(threshold, noise_floor * DEFAULT_THRESHOLD_FACTOR)
        
        if pico > 0.01:
            status_str = "🔥 PALMA!" if (pico > dynamic_thresh and (atual / pico) < DECAY_RATIO) else "   ruído"
            print(f"[{status_str}] Pico: {pico:.3f} | Dinam.Thresh: {dynamic_thresh:.3f} | Queda: {(atual/pico if pico > 0 else 1):.2f}")
            
        if pico > dynamic_thresh and (atual / pico) < DECAY_RATIO:
            now = time.time()
            if (now - last_trigger) > COOLDOWN:
                last_trigger = now
                print("\n>>> [PALMA DETECTADA COM SUCESSO!] <<<\n")

    with sd.InputStream(callback=callback, samplerate=44100, blocksize=2048, channels=1):
        while True:
            time.sleep(0.5)

def run_daemon(threshold):
    print(f"Iniciando serviço de detecção de palmas. Threshold base: {threshold:.4f}")
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
        
        noise_floor = np.mean(history[:2])
        dynamic_thresh = max(threshold, noise_floor * DEFAULT_THRESHOLD_FACTOR)
        
        if pico > dynamic_thresh and (atual / pico) < DECAY_RATIO:
            now = time.time()
            if (now - last_trigger) > COOLDOWN:
                last_trigger = now
                print(f"[CLAP] Palma legítima detectada ({pico:.3f} > {dynamic_thresh:.3f}). Abrindo apps...")
                subprocess.Popen(LAUNCH_CMD, shell=True, start_new_session=True)

    with sd.InputStream(callback=callback, samplerate=44100, blocksize=2048, channels=1):
        while True:
            time.sleep(1)

if __name__ == "__main__":
    threshold = 0.08
    if "--calibrate" in sys.argv:
        threshold = calibrate_mic()
    elif "--threshold" in sys.argv:
        try:
            idx = sys.argv.index("--threshold")
            threshold = float(sys.argv[idx + 1])
        except (IndexError, ValueError):
            print("Erro ao ler o argumento --threshold. Usando padrão 0.08")
            
    if "--test" in sys.argv:
        try:
            run_test(threshold)
        except KeyboardInterrupt:
            print("\nSaindo do modo de teste.")
    else:
        run_daemon(threshold)
