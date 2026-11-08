# ADR 0001, STACK-kysymykset evaluoidaan palvelimella Maximalla deterministisellä siemenellä ja staattisella varatiellä

**Tila.** Hyväksytty 2026-11-07.

## Tilanne

STACK-kysymysten tehtävänanto `questiontext`, malliratkaisu `generalfeedback`, mallivastaukset `tans` ja `[[jsxgraph]]`-kuvaajat riippuvat Maxima-koodista `questionvariables`. Koodi arpoo satunnaismuuttujia funktioilla `rand`, `rand_with_step`, `rand_with_prohib` ja `rand_selection`, laskee matriiseja ja muodostaa LaTeX-kaavoja.

Päätettävänä on kaksi asiaa:

1. Missä ja miten Maxima-koodi ajetaan, kun käyttöliittymä toimii selaimessa WebAssemblyn päällä?
2. Miten esikatselu käyttäytyy, jos `/usr/bin/maxima` puuttuu tai lauseke epäonnistuu, ja miten testit toistavat saman kysymysversion?

## Vaihtoehdot

1. **Pelkkä staattinen tekstikorvaus selaimessa.** Selain korvaa `[[input:...]]`-paikkamerkit ja jättää `{@...@}`-lausekkeet sellaisinaan näkyviin. Tämä ei vaadi Maximaa, mutta kaavat, matriisien koot, valintalistat ja JSXGraph-kuvaajat jäävät keskeneräisiksi.
2. **Palvelinpuolen Maxima-evaluointi ilman varatietä.** Palvelin ajaa `/usr/bin/maxima`-prosessin ja palauttaa virheen, jos Maxima puuttuu tai lauseke epäonnistuu.
3. **Palvelinpuolen `/usr/bin/maxima`-evaluointi eksplisiittisellä `seed: u32`-parametrilla ja automaattisella staattisella varatiellä `render_stack_question_static`.**

## Päätös

Valitaan vaihtoehto 3.

## Seuraukset

- **Toistettavuus säännön 4 mukaisesti.** Funktio `evaluate_stack_question_maxima(q, seed)` ottaa aina parametrin `seed: u32`, ja Maxima-skripti alkaa komennolla `set_random_state(make_random_state(seed))`. Sama kysymys ja sama siemen tuottavat aina samat muuttujat ja mallivastaukset sekä yksikkötesteissä että selaimessa.
- **Vikietoisuus säännön 7 mukaisesti.** Jos `/usr/bin/maxima` ei ole saatavilla tai ajo epäonnistuu, `evaluate_stack_question_maxima` ja `app::api` käyttävät funktiota `render_stack_question_static` ja asettavat kentän `maxima_ok` arvoon `false`. Opettaja näkee silti kysymyksen rakenteen, syötekentät, vastauspuut ja tilamerkin `Staattinen esikatselu`.
- **Yksi prosessikäynnistys per kysymysversio.** Funktio kokoaa kysymyksen muuttujat, syötteiden mallivastaukset, matriisien dimensiot, monivalintojen vaihtoehdot, `[[if test="..."]]`-ehdot sekä CASTextin `{@...@}`- ja `{#...#}`-lausekkeet yhteen Maxima-ajoon.
