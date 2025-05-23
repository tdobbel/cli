import requests
from bs4 import BeautifulSoup

resp = requests.get("https://www.uclouvain.be/fr/resto-u/d-un-pain-a-l-autre-lln")
scrapper = BeautifulSoup(resp.text, "html.parser")
div = scrapper.find("div", attrs={"id": "Nos soupes"})
print("Voici les soupes de la semaine 🍲:")
for p in div.find_all("p"):
    print("*", p.text.strip())
