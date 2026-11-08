# Havainnot, Stackman

Nämä havainnot on kirjattu XML-aineiston, oppaiden `minimal-maxima.pdf` ja `STACK-Guide 2019.pdf` sekä testi-Moodlen `http://10.134.240.190/` tarkastuksen jälkeen.

## Tilanne

Opettajilla on satoja Moodle XML -tiedostoihin vietyjä matematiikan ja diskreetin matematiikan tehtäviä vuosilta 2016 ja 2026. XML-tiedostojen tarkastaminen tekstieditorissa on hidasta, koska tehtävänannot ovat CDATA-lohkoissa täynnä LaTeXia, Maxima-muuttujia, ehtolauseita ja Base64-kuvia. Moodlessa yhden kysymyksen tarkastaminen vaatii koko XML-tiedoston tuonnin kysymyspankkiin ja jokaisen tehtävän avaamisen erikseen.

## Aineistosta löytyneet ongelmat

1. **Kolme eri XML-rakennetta.** Virallinen Moodle-vienti alkaa `<quiz>`-juurielementillä. Osa tiedostoista sisältää pelkän yksittäisen `<question>`-elementin. Kansion `material/STACK_*` tiedostot käyttävät lisäksi yksinkertaistettua rakennetta, jossa `<name>` sisältää suoraan tekstin ilman `<text>`-alitägiä ja vastaustestit ovat muodossa `<answertests><answertest>AlgEquiv</answertest></answertests>`.
2. **Maxima-koodin rivinvaihdot ilman puolipistettä.** Tiedostossa `johdatus_graafiteoriaan.xml` kenttä `questionvariables` sisältää useita sijoituksia eri riveillä ilman `;`- tai `$`-lopetusmerkkiä. Maxima hylkää tällaisen syötteen syntaksivirheenä, ellei esikäsittelijä päätä lauseita rivinvaihtojen kohdalla silloin, kun sulkusyvyys on nolla.
3. **Maximan varattu sana `step`.** STACKin apufunktio `rand_with_step` ei voi käyttää parametrin nimenä sanaa `step`, koska `step` on Maximan silmukoiden varattu avainsana. Kun parametrin nimeksi vaihdetaan `st`, kaikki 188 kansion `material/` kysymystä ja kaikki 39 tiedoston `tentti-Test-top-20261107-1344.xml` STACK-kysymystä evaluoituvat virheittä.

## Havainnoinnin tulokset

| Kysymys | Havainto |
|---------|----------|
| Miten vuoden 2016 ja vuoden 2026 XML-viennit eroavat? | Moodle 4+ ja uudempi STACK lisäävät elementit `<idnumber>`, `<questiondescription>`, `<decimals>`, `<scientificnotation>`, `<logicsymbol>`, `<isbroken>`, vastauspuun `<feedbackstyle>`, solmun `<description>` sekä vastaustestit `Bool` ja `SetEquiv`. Tietomalli määrittelee nämä valinnaisiksi kentiksi, jotta molemmat vuosikerrat toimivat. |
| Mitä erikoisuuksia CASTextissä esiintyy? | Merkintä `{@expr@}` voi viitata joko matemaattiseen lausekkeeseen tai Maxima-merkkijonoon, kuten `solmu: "pienin"`. Jos Maximan `stringp(val)` on tosi, arvoa ei kääritä `\(...\)`-matematiikkatilaan eikä muunneta `tex1`-funktiolla. Lisäksi osa `{@...@}`-merkinnöistä on jo valmiiksi `\(...\)`- tai `\begin{align*}`-ympäristön sisällä. |
| Miten `[[jsxgraph]]` välittää muuttujat? | Lohko luo `<div>`-elementin, jonka DOM-tunniste annetaan JavaScript-funktiolle parametrina `divid`. Lohkon sisällä olevat `{#var#}`-viittaukset korvataan Maximan `string(val)`-tulosteella ennen kuin selain suorittaa kuvaajakoodin. |
| Miten XML-tiedoston kuvat toimivat? | Kysymystekstissä on `<img src="@@PLUGINFILE@@/kuva.png">` ja saman `<questiontext>`-elementin alla `<file name="kuva.png" encoding="base64">`. Renderöijä korvaa `@@PLUGINFILE@@/kuva.png`-osoitteen suoraan `data:image/png;base64,...`-muotoisella osoitteella. |

## Avoimet oletukset

Jos tehtävä riippuu näistä kohdista ja vaatii muuta kuin oletusta, agentin on pysähdyttävä ja kysyttävä ihmiseltä.

| Avoin kysymys | Oletus |
|---------------|--------|
| Ajetaanko opiskelijan syöttämä vastaus vastauspuun läpi Maximassa? | Ei speceissä 001 ja 002. Esikatselu näyttää opettajan mallivastaukset sekä vastauspuun solmujen ehdot, pisteet ja palautteet. |
| Mitä rajapintaa Moodle-integraatio käyttää specissä 003? | Palvelin käyttää Moodlen kysymyspankin XML-tuonti- ja vientiväylää tai Web Service -kutsuja palvelinpuolen istunnolla. |
| Muokataanko kysymystä specissä 004 lomakkeella vai raakana XML-tekstinä? | Käyttöliittymä tarjoaa lomakekentät keskeisille osille, kuten nimelle, kysymystekstille, Maxima-muuttujille, syötteiden mallivastauksille ja yleiselle palautteelle, ja säilyttää muun XML-rakenteen ennallaan. |

## Koeajon havainnot

Specit 001 ja 002 ajettiin oikeaa XML-aineistoa vasten komennoilla `cargo test -p stack` ja `cargo leptos build`:

- **Yksinkertaistetun XML-muodon vastaustestit.** Kirjasto `quick-xml` ei deserialisoi `Vec<AnswerTest>`-kenttää suoraan `<answertest>AlgEquiv</answertest>`-tekstielementeistä. Apurakenne `SimpleAnswerTests` lukee arvot `Vec<String>`-muotoon ja muuntaa ne `AnswerTest`-arvoiksi.
- **Maximan varattu avainsana.** Funktio `rand_with_step(lower, upper, st)` käyttää parametrinimeä `st` sanan `step` sijaan.
- **Leptos 0.8 ja `Suspense`-rajan tyyppisyvyys.** Kun `cargo test --workspace` kääntää työtilan, Cargo aktivoi yhtä aikaa ominaisuudet `ssr` ja `hydrate`. Syvästi sisäkkäiset näkymätyypit `<Suspense>`-komponentin sisällä ylittivät Rustin oletusrekursiorajan `Send`-tarkistuksessa. Alikomponentit käyttävät `.into_any()`-kutsua ja `crates/app/src/lib.rs` asettaa rajan `#![recursion_limit = "256"]`.
