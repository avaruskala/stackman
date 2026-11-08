# Tilaus, Stackman

Tämä muistio sisältää opettajan alkuperäisen tilauksen ja tarkentavat vastaukset. Tulkinnat ja säännöt ovat tiedostossa `domain.md`.

## Tilaus

> Tarvitsen Leptos-pohjaisen web-sovelluksen, johon voin ladata Moodle XML -tiedostoja, jäsentää ne ja selata kysymyksiä vasemman reunan listasta niin, että valitun kysymyksen renderöity esikatselu näkyy rinnalla oikealla.
>
> Käytä olemassa olevaa `stack`-cratea ja oppaiden `material/minimal-maxima.pdf` sekä `material/STACK-Guide 2019.pdf` sääntöjä. Ota tarvittaessa mallia Moodlen ja STACKin lähdekoodista siinä, miten kysymysteksti, syötteet ja palautteet renderöidään.
>
> Testaamista varten on paikallinen Moodle-instanssi osoitteessa `http://10.134.240.190/`. Koodin `crates/app`-kansiossa pitää olla selkeää, luettavaa ja ylläpidettävää.

## Tarkennukset

| Kysymys | Vastaus |
|---------|---------|
| Miten STACK-kysymysten Maxima-muuttujat ja CASText käsitellään? | "Palvelin ajaa ne oikealla Maximalla, ja jos Maxima ei ole saatavilla tai lauseke epäonnistuu, sovellus näyttää staattisen esikatselun kaatumatta." |
| Mitä kysymystyyppejä XML-tiedostoissa on? | "Pääasiassa STACK-kysymyksiä, mutta samoissa tenttivienneissä on myös Moodlen monivalintoja, tosi tai epätosi -kysymyksiä ja kategoriaotsikoita." |
| Mitä esikatselun pitää näyttää? | "Kysymysteksti syötekenttineen, painike jolla saa täytettyä opettajan oikeat mallivastaukset, malliratkaisu, muuttujien arvotaulukko ja vastauspuut." |
| Miten matemaattiset kaavat ja kuvaajat piirretään? | "MathJax 3 hoitaa LaTeX-kaavat ja JSXGraph piirtää `[[jsxgraph]]`-kuvaajat samalla tavalla kuin Moodlessa." |
| Miten monikieliset kysymykset näytetään? | "Työkalupalkissa on valitsin, jolla voi näyttää kaikki kielet tai suodattaa pelkän suomen tai englannin." |
| Miten satunnaistetut kysymysversiot toimivat? | "Käyttäjä voi syöttää siemenen, arpoa uuden version painikkeesta tai valita kysymyksen käyttöönotetuista versioista." |
| Miten kysymyksiä siirretään ja muokataan? | "Opettajan pitää pystyä tuomaan kysymyksiä testi-Moodlesta osoitteesta `http://10.134.240.190/`, muokkaamaan niitä ja viemään ne takaisin Moodle XML -muotoon ilman että kenttiä katoaa." |

## Havainnoitaessa selvitettävät

Asiakas antoi XML-aineiston kansioissa `material/` ja `tentti-Test-top-20261107-1344.xml` sekä oppaat ja pyysi selvittämään:

- Miten vuoden 2016 ja vuoden 2026 Moodle XML -viennit eroavat toisistaan?
- Mitä syntaksipoikkeamia oikeiden XML-tiedostojen Maxima-koodissa ja CASTextissä esiintyy?
- Miten `[[jsxgraph]]`-lohkot saavat muuttujien arvot Maximasta JavaScriptiin?
- Miten XML-tiedostoon upotetut Base64-kuvat näytetään selaimessa?

## Myöhemmin selvinneet kysymykset

| Kysymys | Vastaus |
|---------|---------|
| Miksi osa kansion `material/STACK_*` tiedostoista ei ala `<quiz>`-elementillä? | Tiedostot ovat yksinkertaistettuja `<question type="stack">`-esimerkkejä. Jäsentäjä tukee molempia muotoja säännön 1 mukaisesti. |
| Ajetaanko opiskelijan syöttämä vastaus vastauspuun läpi Maximassa? | Ei tässä versiossa. Esikatselu näyttää opettajan mallivastaukset ja vastauspuun solmut. |
