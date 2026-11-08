# ADR 0002, Moodle XML -tietomalli säilyttää sekä vuoden 2016 että vuoden 2026 Moodle 4+ -kentät

**Tila.** Hyväksytty 2026-11-07.

## Tilanne

Aineistossa on kolmenlaisia XML-tiedostoja:

1. Vuoden 2016 Moodle- ja STACK-viennit kansiossa `material/tentti-Matematiikan opettajat-Discrete Mathematics (TTY)-20160515-1723/`.
2. Yksinkertaistetut `<question type="stack">`-tiedostot kansiossa `material/STACK_*`.
3. Vuoden 2026 Moodle 4+ -viennit tiedostossa `tentti-Test-top-20261107-1344.xml`, joissa on uudet elementit `<idnumber>`, `<questiondescription>`, `<decimals>`, `<scientificnotation>`, `<logicsymbol>`, `<isbroken>`, vastauspuun `<feedbackstyle>`, solmun `<description>` sekä vastaustestit `Bool` ja `SetEquiv`.

Jos tietomalli kansiossa `crates/stack/src/models/` sisältää vain vanhat kentät, uudet XML-tiedostot joko epäonnistuvat jäsennyksessä tai menettävät uudet kenttänsä XML-viennissä. Jos taas uudet kentät ovat pakollisia, vuoden 2016 XML-tiedostot eivät jäsenny.

## Vaihtoehdot

1. **Kevyt esikatselumalli.** Poimitaan XML-tiedostosta vain esikatselun tarvitsemat kentät ja jätetään muut kentät huomiotta.
2. **Häviötön `stack::models`-tietomalli, jossa Moodle 4+ -lisäkentät ovat valinnaisia `Option<T>`-kenttiä.** Yksinkertaistettu `STACK_*`-muoto muunnetaan jäsennyksessä `SimpleStackQuestionXml`-rakenteesta tavalliseksi `StackQuestion`-olioksi.

## Päätös

Valitaan vaihtoehto 2.

## Seuraukset

- Sekä vuoden 2016 että vuoden 2026 Moodle XML -tiedostot deserialisoituvat ja serialisoituvat takaisin XML-muotoon funktiolla `quick_xml::se::to_string` ilman tietohävikkiä säännön 13 mukaisesti.
- Funktio `parse_moodle_xml` muuntaa yksinkertaistetut `material/STACK_*`-tiedostot samaksi `Quiz`-rakenteeksi, joten käyttöliittymä ja CAS-evaluaattori käsittelevät kaikkia STACK-kysymyksiä yhdellä koodipolulla.
- Kun opettaja muokkaa kysymystä specissä 004, muutos päivittää kohdekentät suoraan tietomalliin ja säilyttää Base64-kuvaliitteet sekä vastauspuut ennallaan.
