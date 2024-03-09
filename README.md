## Käyttö
Joudut itse kääntämään ohjelman :-)

Ohjelma vaatii kolme tiedostopolkua komentoriviargumentteina, tässä järjestyksessä:
- Päiväkirja-excel Holvista
- tilikarttatiedosto
  - jos tiedostopääte on .tlk käsitellään tiedostoa Tappion tilikarttatiedostoformaattina, ja sen tilitapahtumalista ylikirjoitetaan Holvista tuoduilla
  - muussa tapauksessa tiedostoa käsitellään jäjelmpänä kuvaillun tilikarttaformaatin mukaisesti
- tiedostopolku, johon valmis Tappio-tiedosto kirjoitetaan

### Tilikarttatiedostoformaatin kuvaus
Tiedoston alussa on kolme riviä, joista ensimmäinen kertoo tilikauden otsakkeen. Seuraavat kaksi ovat päivämääriä muodossa `d.m.y`, jotka kertovat tilikauden avaus- ja lopetuspäivän. Näitä rivejä seuraa yksi tyhjä rivi ja tilikarttamäärittely.

Tilikarttamäärittelyssä kukin tili määritellään omalla rivillään. Tyhjiä rivejä ei sallita. Rivi alkaa sisennyksellä, joka voi olla nolla tai useampi tulostumaton (_witespace_) merkki (ei rivinvaihto). Perustason tilit (Vastaavaa, Vastattavaa, Tulos) ovat sisentämättömiä. Tilin alatilit on lueteltu sen alle suuremmalla sisennystasolla. Tilin kaikkien alatilien tulee olla samalla sisennystasolla.

Sisennystä seuraa muotoa `-?\d+` oleva tilin numero, tilin nimi ja mahdollinen alkusaldo, kaikki riviväleillä eroteltuna. Alkusaldo on muotoa `-?\d+[.,]?\d*[€$]`, eli (mahdollisesti desimaaliosan sisältävä) lukuarvo jota seuraa €- tai $-merkki ilman välimerkkejä. Desimaalierottimena saa käyttää sekä pistettä että pilkkua, vaikka sekaisin samassa tiedostossa.

Esimerkkitilikarttamäärittely ehkä selventää asiaa:
```
LPK:n kirjanpito 1970
1.1.1970
31.12.1970

-1 Vastaavaa
  100 Rahat
    110 Pankkitilit
      111 Pankkitili ilman alkusaldoa
      112 Pankkitili alkusaldolla 15000€
    120 Käteiskassa 20.0€
-1 Vastattavaa
  200 Siirtovelat -12,34$
-1 Tulos
  300 Partiotoiminta
    310 Sudenpentutoiminta
      311 Retkien menot
      312 Retkien tulos
    320 Muu partiotoiminta
```
