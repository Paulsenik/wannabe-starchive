#!/bin/bash

# Prüfen, ob Python installiert ist
if ! command -v python3 &> /dev/null
then
    echo "Python3 ist nicht installiert. Bitte installiere Python3, um fortzufahren."
    exit 1
fi

echo "Erstelle virtuelle Umgebung im aktuellen Verzeichnis (.venv)..."
python3 -m venv .venv

echo "Aktiviere die virtuelle Umgebung..."
source .venv/bin/activate

echo "Installiere 'youtube-transcript-api'..."
pip install youtube-transcript-api

echo "Setup abgeschlossen. Die virtuelle Umgebung ist aktiviert."
echo "Um sie zu verlassen, gib 'deactivate' ein."
echo "Um sie erneut zu aktivieren, navigiere in das Projektverzeichnis und gib 'source .venv/bin/activate' ein."