# Spec 004, Kysymyksen muokkaus ja häviötön Moodle XML -vienti

## Tarina

Opettajana haluan muokata valitun kysymyksen nimeä, tehtävänantoa, Maxima-muuttujia, syötteiden mallivastauksia ja yleistä palautetta, nähdä muutoksen vaikutuksen heti esikatselussa sekä viedä muokatun kysymyspankin takaisin Moodle XML -tiedostoksi ilman, että Moodle 4+ -kentät tai Base64-kuvaliitteet katoavat.

## Käsitteet

Sanasto ja kentät on määritelty tiedostossa `docs/domain.md`.

## Tekemiset

| Toiminto | Onnistumisen ehto | Epäonnistumisen seuraus |
|----------|-------------------|-------------------------|
| Muokkaa kysymystä ja esikatsele | Opettaja avaa muokkaustilan, muuttaa esimerkiksi `questionvariables`- tai `questiontext`-kenttää ja tallentaa muutoksen muistiin. Esikatselu ja Maxima-evaluointi päivittyvät heti. | Jos kysymyksen nimi on tyhjä tai `[[input:X]]` viittaa puuttuvaan syötteeseen, tarkistaja hylkää tallennuksen ja näyttää virheen säännön 14 mukaisesti. |
| Vie kysymyspankki XML-tiedostoksi | Kaikki kysymykset läpäisevät säännön 14 tarkistuksen, ja sovellus serialisoi `Quiz`-olion Moodle XML -tiedostoksi. | Jos yksikin kysymys rikkoo sääntöä 14, sovellus estää viennin ja ilmoittaa virheellisen kysymyksen. |

## Säännöt ja paikat

Viittaukset tiedoston `docs/domain.md` sääntöihin:

- Sääntö 1, viedyn XML-tiedoston on jäsennyttävä takaisin funktiolla `parse_moodle_xml`, sijaitsee funktiossa `stack::render::parse_moodle_xml`.
- Sääntö 13, Moodle 4+ -kentät ja `<file encoding="base64">`-liitteet säilyvät muuttumattomina, sijaitsee moduulissa `stack::models` ADR 0002:n mukaisesti.
- Sääntö 14, tyhjä kysymyksen nimi tai määrittelemätön `[[input:X]]`-viittaus hylätään, sijaitsee moduulissa `stack::models`.

## Hyväksymiskriteerit

1. **Alkutila:** Tiedosto `tentti-Test-top-20261107-1344.xml` on jäsennetty `Quiz`-olioksi. **Toiminto:** `Quiz` serialisoidaan XML-merkkijonoksi funktiolla `quick_xml::se::to_string` ja jäsennetään uudelleen funktiolla `parse_moodle_xml`. **Lopputulos:** Uudelleen jäsennetty `Quiz` on yhtäsuuri alkuperäisen kanssa, mukaan lukien kentät `<idnumber>`, `<questiondescription>`, `<feedbackstyle>` ja `<file encoding="base64">`-liitteet säännön 13 mukaisesti.
2. **Alkutila:** Valittuna on STACK-kysymys, jonka `questionvariables` on `a: 2;`. **Toiminto:** Opettaja muuttaa kentän arvoon `a: 7;` ja tallentaa. **Lopputulos:** Esikatselun Maxima-evaluointi päivittyy ja näyttää muuttujan `a` arvona `7`.
3. **Alkutila:** Opettaja muuttaa kysymyksen nimeksi tyhjän merkkijonon `""` tai pelkkiä välilyöntejä. **Toiminto:** Opettaja yrittää tallentaa kysymyksen tai viedä sen XML-muotoon. **Lopputulos:** Tarkistaja hylkää toiminnon virheellä `"Kysymyksen nimi ei saa olla tyhjä"` säännön 14 mukaisesti.
4. **Alkutila:** STACK-kysymyksen `questiontext` sisältää paikkamerkin `[[input:ans99]]`, mutta `<input>`-listassa on vain `ans1`. **Toiminto:** Kysymys tarkistetaan vientiä varten. **Lopputulos:** Tarkistaja hylkää viennin virheellä, joka nimeää puuttuvan syötteen `ans99` säännön 14 mukaisesti.

Kriteerit 1, 3 ja 4 ajetaan craten `crates/stack` yksikkötesteinä.

## Ei-tavoitteet

- Sovellus ei sisällä WYSIWYG-tekstieditoria, vaan opettaja muokkaa CASTextiä ja Maxima-koodia tekstikentissä.
- Sovellus ei sisällä kuvankäsittelyä Base64-liitteille.

## Tehtävät

1. **Stack-työntekijä cratessa `crates/stack`.** Toteuta kysymyksen tarkistusfunktiot `validate_question` ja `validate_quiz` säännön 14 mukaisesti sekä XML-vientifunktio `export_moodle_xml` säännön 13 mukaisesti. Askel on valmis, kun kriteerit 1, 3 ja 4 menevät läpi komennolla `cargo test -p stack`.
2. **App-työntekijä kansiossa `crates/app/src/components/preview/`.** Toteuta muokkauslomake ja välitön uudelleenevaluointi tallennuksen yhteydessä. Askel on valmis, kun kriteeri 2 toimii ja esikatselu päivittyy.
3. **App-työntekijä tiedostoissa `crates/app/src/components/header.rs` ja `crates/app/src/api.rs`.** Toteuta Vie XML -painike, joka lataa tarkistetun Moodle XML -tiedoston selaimesta. Askel on valmis, kun muokattu XML latautuu ja jäsentyy uudelleen virheittä.
