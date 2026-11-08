# Domain, Stackman

Tämä tiedosto määrittelee sovelluksen sanaston, käsitteet ja säännöt tarkistuspaikkoineen.

## Sanasto

Taulukossa ovat asiakkaan ja STACK-oppaiden termit. Sarake Koodissa määrittelee vastaavan Rust-tunnisteen.

| Sana | Koodissa | Kuvaus |
|------|----------|--------|
| Kysymyspankki | `Quiz` | Moodle XML -tiedoston juurikokoelma `<quiz>`, joka sisältää kategoriaotsikoita ja kysymyksiä. |
| Kysymys | `Question` | Yksi XML-tiedoston alkio: `Stack`, `Multichoice`, `TrueFalse` tai `Category`. |
| Kysymysrivi | `QuestionEntry` | Käyttöliittymän suodatettava rivi, joka yhdistää kysymyksen, lähdetiedoston ja siivotun kategorian. |
| STACK-kysymys | `StackQuestion` | CAS-pohjainen matematiikkatehtävä, jolla on kysymysmuuttujat, CASText-tekstit, syötteet ja vastauspuut. |
| Kysymysmuuttujat | `questionvariables` | Maxima-koodilohko, joka alustaa ja laskee kysymyksen muuttujat valitulla siemenellä. |
| Siemen | `seed` | Positiivinen kokonaisluku `u32`, joka alustaa Maximan satunnaisgeneraattorin. |
| CASText | `castext` | HTML- ja LaTeX-teksti, joka sisältää Maxima-lausekkeita, ehtolohkoja, syötteitä ja kuvaajia. |
| Syöte | `Input` / `EvaluatedInput` | Opiskelijan vastauskenttä ja siihen kuuluva opettajan mallivastaus. |
| Mallivastaus | `tans` | Opettajan määrittelemä Maxima-lauseke oikealle vastaukselle. |
| Vastauspuu | `Prt` / `EvaluatedPrt` | Suunnattu verkko solmuja, jotka testaavat opiskelijan vastausta ja laskevat pisteet sekä palautteen. |
| Vastauspuun solmu | `Node` / `EvaluatedPrtNode` | Yksi tarkistusaskel, joka vertaa opiskelijan vastausta ja mallivastausta vastaustestillä. |
| Evaluoitu STACK-kysymys | `EvaluatedStackQuestion` | Yhden STACK-kysymyksen ja yhden siemenen laskettu esikatselunäkymä. |

## Käsitteet

**Kysymyspankki.** Lista `Question`-alkioita tiedoston mukaisessa järjestyksessä.

**STACK-kysymys.** Nimi, kysymysteksti, yleinen palaute, oletuspisteet, vähennys, kysymysmuuttujat, kysymysmuistiinpano, syötteet, vastauspuut, käyttöönotetut siemenet ja tägit.

**Syöte.** Nimi, syötetyyppi `InputType`, mallivastaus `tans`, kentän leveys `boxsize`, syntaksivihje `syntaxhint`, matriisin solut ja monivalinnan vaihtoehdot.

**Vastauspuu.** Nimi, pistearvo `value`, automaattinen sievennys `autosimplify`, palautemuuttujat `feedbackvariables` ja solmut `node`.

**Evaluoitu STACK-kysymys.** Siemen `seed`, Maxima-suorituksen tila `maxima_ok`, mahdollinen virheviesti `maxima_error`, renderöity kysymysteksti, yleinen palaute, kysymysmuistiinpano, evaluoidut muuttujat, evaluoidut syötteet ja vastauspuut.

## Invariantit

Tarkistuspaikan ratkaisee se, mitä tietoa sääntö tarvitsee. Puhdas XML- ja CAS-logiikka kuuluu crateen `crates/stack`. Käyttöliittymän suodatus ja palvelimen tiedostopolut kuuluvat crateen `crates/app`.

| # | Sääntö | Paikka | Perustelu |
|---|--------|--------|-----------|
| 1 | Jäsentäjä hyväksyy `<quiz>`-kokoelman, yksittäisen `<question>`-elementin ja yksinkertaistetun `<question type="stack">`-muodon, ja hylkää rikkinäisen tai tuntemattoman XML-syötteen. | `stack::render::parse_moodle_xml` | Jäsentäjä tuntee kaikki kolme XML-muotoa, eikä rikkinäinen XML saa luoda vajaata kysymyspankkia. |
| 2 | `category`-tyyppinen alkio ei luo kysymysriviä, vaan asettaa siivotun kategorian sitä seuraaville kysymyksille. | `app::models::extract_questions_from_quiz` | Kategoria toimii XML-virrassa otsikkona seuraaville kysymyksille. |
| 3 | Jokaisella STACK-syötteellä on `InputType` ja mallivastaus `tans`, ja matriisisyötteen rivit sekä sarakkeet johdetaan Maximan `matrix`-tuloksesta. | `stack::render::evaluate_stack_question_maxima` | Syötteen tyyppi ja matriisin koko kuuluvat kysymyksen ja CAS-evaluoinnin rakenteeseen. |
| 4 | Funktio ottaa siemenen `seed: u32` parametrina ja alustaa Maximan komennolla `set_random_state(make_random_state(seed))` ennen kysymysmuuttujien suoritusta. | `stack::render::build_maxima_script` | Sama siemen tuottaa aina saman tehtäväversion ilman kelloriippuvuutta, kuten ADR 0001 määrää. |
| 5 | Esikäsittelijä poistaa `/* ... */`-kommentit ja päättää `questionvariables`-lauseet puolipisteeseen, dollariin tai rivinvaihtoon vain silloin, kun sulkusyvyys on nolla eikä rivi pääty jatko-operaattoriin. | `stack::render::normalize_maxima_statements` | Moniriviset matriisit ja funktiot pysyvät ehjinä, mutta puolipisteettömät rivit saavat Maximan vaatiman päätteen. |
| 6 | Renderöijä tulostaa `{@expr@}`-lausekkeen raakana tekstinä, jos Maximan arvo on merkkijono, ja käärii LaTeX-kaavan `\(...\)`-merkintään vain silloin, kun kohta ei ole jo valmiiksi matematiikkatilassa. | `stack::render::render_castext` | Merkkijonomuuttujat ovat tavallista tekstiä, ja sisäkkäinen matematiikkatila rikkoo MathJax-ladonnan. |
| 7 | Jos Maxima-suoritus epäonnistuu tai aikakatkaistaan, evaluoija palauttaa staattisen esikatselun `render_stack_question_static`, asettaa kentän `maxima_ok` epätodeksi ja tallentaa virhesyyn. | `stack::render` ja `app::api` | Yksi virheellinen CAS-lauseke tai puuttuva Maxima-asennus ei saa kaataa esikatselua, kuten ADR 0001 määrää. |
| 8 | Vastauspuun solmun `truenextnode` ja `falsenextnode` ovat joko seuraavan solmun indeksi tai lopetusarvo `-1`. | `stack::models::stack::Node` | Solmun oma rakenne määrää puun jatkumisen tai päättymisen. |
| 9 | Monivalintakysymyksen `single = true` sallii yhden valinnan ja `single = false` useita valintoja, ja oikeita vaihtoehtoja ovat ne, joiden `fraction` on suurempi kuin nolla. | `stack::models::multichoice` ja `app::components::preview::multichoice` | Kysymyksen omat kentät määräävät valintatavan ja oikeat vastaukset. |
| 10 | Tosi tai epätosi -kysymyksessä vaihtoehto on oikea silloin, kun sen `fraction` on suurempi kuin nolla. | `stack::models::truefalse` | Kysymyksen oma kenttä määrää oikean vastauksen. |
| 11 | Palvelimen esimerkkilataus `load_sample_xml_file` hyväksyy vain polun `tentti-Test-top-20261107-1344.xml` tai kansion `material/` polut ja hylkää jokaisen polun, joka sisältää merkit `..`. | `app::api::load_sample_xml_file` | Palvelin tarkistaa polun ennen tiedoston lukemista. |
| 12 | Moodle-instanssin salasanaa tai tokenia ei tallenneta repoon eikä lähetetä selaimelle, vaan yhteys kulkee palvelimen kautta. | `app::api` ja `crates/server` | Tunnistautumistiedot pysyvät palvelimella. |
| 13 | XML-vienti säilyttää kaikki Moodle 4+ -kentät ja Base64-tiedostoliitteet muuttumattomina. | `stack::models` | Kysymyksen muokkaus ja vienti ei saa hukata Moodlen metatietoja eikä kuvia, kuten ADR 0002 määrää. |
| 14 | Kysymyksen tallennus ja vienti hylätään, jos kysymyksen nimi on tyhjä tai kysymystekstin `[[input:X]]`-paikkamerkille ei ole vastaavaa `<input>`-määrittelyä. | `stack::models` | Tarkistus estää virheellisen Moodle XML -tiedoston viennin. |
| 15 | CASText-renderöijä jäsentää STACK-JS- ja apulohkot (`[[comment]]`, `[[escape]]`, `[[template]]`, `[[entityescape]]`, `[[jsstring]]`, `[[quid]]`, `[[cors]]`, `[[style]]`, `[[script]]`, `[[iframe]]`, `[[javascript]]`, `[[jsxgraph]]`), tuottaa virheilmoituksen irrallisille `[[script]]`- ja `[[style]]`-lohkoille sekä käärii `[[iframe]]`-, `[[javascript]]`- ja `[[jsxgraph]]`-lohkot eristettyihin `stack-iframe-holder`-elementteihin suorittamatta raakoja `<script>`-tageja pääikkunassa. | `stack::render::render_castext` ja `stack::render::collect_castext_expressions` | STACK-JS-arkkitehtuuri eristää skriptit hiekkalaatikkokehyksiin ja välittää syötetapahtumat `postMessage`-viesteillä. |

## Arvo vai perustyyppi

**Suodattimet `QuestionTypeFilter` ja `LanguageFilter` ovat enum-arvoja.** Kääntäjä varmistaa, ettei suodattimeen voi asettaa tuntematonta merkkijonoa.

**`AnswerTest` ja `InputType` ovat enum-arvoja.** Vain STACKin tukemat vastaustestit ja syötetyypit ovat sallittuja.

**Siemen `seed` on `u32`-luku.** Mikä tahansa 32-bittinen positiivinen kokonaisluku kelpaa Maximan satunnaisgeneraattorin syötteeksi.

## Laskettu näkymä `EvaluatedStackQuestion`

Sovellus ei tallenna `EvaluatedStackQuestion`-rakennetta XML-tiedostoon eikä tietokantaan. Palvelin laskee sen aina uudelleen `StackQuestion`-oliosta ja valitusta `seed: u32`-arvosta. Kun opettaja vaihtaa siementä tai muokkaa kysymysmuuttujia, esikatselu päivittyy suoraan lähdedatasta.

## Käyttäytymiset

| Toiminto | Käynnistäjä | Sitovat säännöt | Rajaus |
|----------|-------------|-----------------|--------|
| Tuo XML-tiedostoja | Opettaja | 1, 2 | Ei muokkaa alkuperäistä tiedostoa levyllä. |
| Lataa esimerkkitiedosto | Opettaja | 1, 2, 11 | Ei lue tiedostoja kansion `material/` ulkopuolelta. |
| Suodata ja valitse kysymys | Opettaja | 2 | Ei käynnistä Maximaa ennen kuin käyttäjä valitsee STACK-kysymyksen. |
| Evaluoi STACK-kysymys siemenellä | Opettaja tai käyttöliittymä | 3, 4, 5, 6, 7, 8, 15 | Ei muuta kysymyksen XML-rakennetta. |
| Näytä mallivastaukset ja palaute | Opettaja | 3, 9, 10 | Ei tallenna opiskelijan vastauksia. |
| Tuo kysymyksiä testi-Moodlesta | Opettaja | 1, 2, 12 | Ei palauta Moodlen salasanaa tai tokenia selaimeen. |
| Muokkaa ja vie XML-tiedostoon | Opettaja | 13, 14 | Ei poista Moodle 4+ -kenttiä eikä Base64-liitteitä. |
