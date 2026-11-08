# Versiohallinta, Stackman

Tämä tiedosto määrittelee haarojen käytön, commit-käytännön ja session lopetusviestin. Agenttien roolit ovat tiedostossa `docs/agentit.md`.

## Haarat

| Haara | Sisältö | Luoja | Yhdistäjä |
|---|---|---|---|
| `master` | Julkaistu haara, joka kääntyy komennolla `cargo leptos build`, läpäisee testit `cargo test --workspace` ja täyttää tiedoston `arkkitehtuuri.md` Valmis-ehdot. | Ihminen | Ihminen pull requestilla haarasta `development` |
| `development` | Kehityshaara, josta ominaisuushaarat lähtevät ja johon ne palaavat. | Ihminen | Ihminen pull requestilla ominaisuushaarasta |
| `feature/NNN-aihe` | Yhden specin toteutushaara, kuten `feature/001-xml-tuonti-ja-kysymyslista`. | Johtaja haarasta `development` | Ihminen yhdistää ja poistaa haaran |

Yksi spec vastaa yhtä ominaisuushaaraa. Pitkäikäisiin haaroihin ei tehdä suoria push-toimenpiteitä, rebasea eikä force pushia.

## Spec ennen koodia

- Uusi ominaisuus alkaa uudella `.ai/specs/NNN-*.md`-tiedostolla, joka commitoidaan ennen ominaisuuden koodimuutoksia.
- Dokumentaation muutos ja koodimuutos tehdään eri commiteissa. Jos sääntö muuttuu, päivitä ensin `domain.md`, sitten `spec` ja lopuksi koodi omina committeinaan.
- Uusi arkkitehtuuripäätös kirjataan uuteen `.ai/docs/adr/NNNN-*.md`-tiedostoon.
- Työntekijä ei muokkaa kansioita `.ai/docs/` tai `.ai/specs/`.

## Askeleen työnkulku

1. Johtaja luo ominaisuushaaran haarasta `development` tai jatkaa olemassa olevaa haaraa.
2. Johtaja antaa yhden specin tehtäväaskeleen yhdelle työntekijälle. Yksi askel vastaa yhtä committia.
3. Katselmoija tarkistaa diffin tiedostojen `domain.md` ja `arkkitehtuuri.md` sääntöjä sekä specin kriteerejä vasten. Työntekijä korjaa löydöt ennen committia.
4. Johtaja varmistaa, että `cargo leptos build` ja `cargo test --workspace` menevät läpi virheittä.
5. Johtaja tekee commitin. Vain johtaja suorittaa git-komentoja.
6. Kun specin kaikki askeleet ovat valmiit, johtaja kirjoittaa lopetusviestin ihmiselle.

## Commit-viesti

Yksi commit sisältää yhden tehtävälistan askeleen sekä siihen kuuluvat testit. Otsikko kuvaa muutoksen asiakkaan kielellä tiedostonimen sijaan. Runko kertoo specin, askeleen, säännöt, ajetut kriteerit ja testituloksen.

Esimerkki:

```text
Jäsennä kolme Moodle XML -muotoa ja evaluoi STACK-muuttujat Maximalla

Spec 001 ja 002, askel 1: parse_moodle_xml ja evaluate_stack_question_maxima
sääntöjen 1 ja 3-8 mukaisesti cratessa crates/stack.
Siemen annetaan parametrina ADR 0001:n mukaisesti.

Kriteerit menevät läpi kaikilla 37 XML-tiedostolla.
cargo test --workspace: Passed 2, Failed 0.
```

## Pull request

Jokaisesta ominaisuushaarasta avataan yksi pull request, jonka otsikko on specin nimi. Runko listaa toteutetut askeleet, ajetut hyväksymiskriteerit tuloksineen sekä mahdolliset ajamatta jääneet kohdat perusteluineen. Ihminen tarkastaa ja yhdistää pull requestin.

## Kielletyt tiedostot repossa

Tiedosto `.gitignore` estää näiden lisäämisen, eikä niitä saa lisätä pakottamalla:

- Käännöstulokset kansioissa `target/` ja `public/pkg/`.
- Salasanat, Moodle-tokenit ja `.env`-tiedostot.
- Koodi, joka ei käänny komennolla `cargo leptos build`, tai rikkinäinen testi.

Jos salaisuus päätyy repoon, vaihda se heti ja ilmoita asiasta lopetusviestin ensimmäisellä rivillä.

## Agentin lopetusviesti

Johtaja päättää session viestiin, joka sisältää seuraavat tiedot tässä järjestyksessä:

1. Haaran nimi ja tieto siitä, onko haara pushattu.
2. Tehdyt commitit tiivisteineen ja otsikoineen.
3. Ajettujen tarkistusten `cargo leptos build` ja `cargo test --workspace` tulokset.
4. Ajamatta jääneet kriteerit ja syy.
5. Korjaamatta jääneet katselmoijan löydöt ja syy.
6. Avoimet kysymykset ihmiselle viittauksineen tiedostoihin `domain.md` tai `havainnot.md`.
7. Kesken jääneen askeleen tila ja ratkaisua odottava kysymys.
