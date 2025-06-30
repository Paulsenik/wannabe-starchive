# pip install youtube_transcript_api

from youtube_transcript_api import YouTubeTranscriptApi

ytt_api = YouTubeTranscriptApi()
srt = ytt_api.fetch("KhPQtXQpiZc")
# srt = YouTubeTranscriptApi.list_transcripts("KhPQtXQpiZc")

#for transcript in srt:
#  print(transcript.translate('de').fetch())

with open("out/transcript.txt",'w') as f:
  for line in srt:
    f.write(f"{line}\n")