import requests
from bs4 import BeautifulSoup

resp = requests.get("https://www.uclouvain.be/fr/resto-u/d-un-pain-a-l-autre-lln")
scrapper = BeautifulSoup(resp.text, "html.parser")
div = scrapper.find("div", attrs={"id": "Nos soupes"})
p = div.find("p")
soups = p.text.strip().split("Soupe")
if len(soups) < 2:
    print("Pas de soupe cette semaine 😭")
    exit(0)
print("Voici les soupes de la semaine 🍲:")
for soup in soups[1:]:
    print(f"* Soupe {soup.strip()}")

