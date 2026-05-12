import requests

url = "https://api.groq.com/openai/v1/chat/completions"
headers = {
    "Authorization": "Bearer YOUR_GROQ_API_KEY",
    "Content-Type": "application/json"
}
data = {
    "model": "llama3-8b-8192",
    "messages": [{"role": "user", "content": "hello"}]
}

resp = requests.post(url, headers=headers, json=data)
print("Status:", resp.status_code)
print("Response:", resp.text)
