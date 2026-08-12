#!/usr/bin/env python3
"""
Clap Launcher - Usando a biblioteca oficial clap-detector com filtragem de frequência (Bandpass)
e detecção robusta de palmas.
"""
import sys
import time
import subprocess
from clapDetector import ClapDetector, printDeviceInfo

# Configurações de comandos para abrir com a palma
LAUNCH_CMD = (
    "x-terminal-emulator -e hermes & "
    "flatpak run com.brave.Browser & "
    "flatpak run com.discordapp.Discord"
)

def run_test():
    print("--- MODO DE TESTE (clap-detector) ---")
    print("Dispositivos de áudio disponíveis:")
    printDeviceInfo()
    print("\nFazendo varredura com filtros de banda (lowcut=200, highcut=3200)...")
    print("Bata palmas ou faça barulhos para testar. Ctrl+C para sair.\n")
    
    threshold_bias = 6000
    lowcut = 200
    highcut = 3200
    
    detector = ClapDetector(inputDevice=-1, logLevel=10)
    detector.initAudio()
    
    last_trigger = 0
    cooldown = 2.0
    
    try:
        while True:
            audio_data = detector.getAudio()
            result = detector.run(thresholdBias=threshold_bias, lowcut=lowcut, highcut=highcut, audioData=audio_data)
            
            if result and len(result) > 0:
                now = time.time()
                if (now - last_trigger) > cooldown:
                    last_trigger = now
                    print(f"\n>>> [PALMA DETECTADA!] (Qtd: {len(result)}) <<<\n")
            
            time.sleep(1 / 60)
            
    except KeyboardInterrupt:
        print("\nSainando do modo de teste.")
    finally:
        detector.stop()

def run_daemon():
    print("Iniciando serviço de detecção de palmas (clap-detector)...")
    
    threshold_bias = 6000
    lowcut = 200
    highcut = 3200
    
    detector = ClapDetector(inputDevice=-1, logLevel=10)
    detector.initAudio()
    
    last_trigger = 0
    cooldown = 2.0
    
    try:
        while True:
            audio_data = detector.getAudio()
            result = detector.run(thresholdBias=threshold_bias, lowcut=lowcut, highcut=highcut, audioData=audio_data)
            
            if result and len(result) > 0:
                now = time.time()
                if (now - last_trigger) > cooldown:
                    last_trigger = now
                    print(f"[CLAP] Palma detectada! Abrindo aplicativos...")
                    subprocess.Popen(LAUNCH_CMD, shell=True, start_new_session=True)
            
            time.sleep(1 / 60)
            
    except KeyboardInterrupt:
        print("\nServiço interrompido.")
    finally:
        detector.stop()

if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] == "--test":
        run_test()
    else:
        run_daemon()
