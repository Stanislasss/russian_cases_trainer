# Russian text-to-speech with Piper (https://github.com/OHF-Voice/piper1-gpl, GPL-3.0).
# Runs as its own service; the backend calls its HTTP API.
FROM python:3.12-slim

RUN pip install --no-cache-dir "piper-tts[http]"

# All Russian voices, so they can be compared; the backend picks one (TTS_VOICE).
RUN python -m piper.download_voices --data-dir /voices \
        ru_RU-irina-medium ru_RU-denis-medium ru_RU-dmitri-medium ru_RU-ruslan-medium

RUN useradd --system tts
USER tts
EXPOSE 5000
CMD ["python", "-m", "piper.http_server", "--data-dir", "/voices", \
     "-m", "ru_RU-irina-medium", "--host", "0.0.0.0", "--port", "5000"]
