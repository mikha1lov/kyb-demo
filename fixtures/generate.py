#!/usr/bin/env python3
import datetime as dt
import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent
OUT = ROOT / "out"
OUT.mkdir(exist_ok=True)
CHROME = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
FOOTER = "Synthetic test document. Fictional data. Not a real document."

STYLE = """
<style>
 body{font-family:Arial,Helvetica,sans-serif;margin:40px;color:#111;font-size:13px}
 h1{font-size:20px;margin:0 0 4px} h2{font-size:15px;margin:18px 0 6px}
 table{border-collapse:collapse;width:100%} td,th{border:1px solid #888;padding:4px 6px;text-align:left}
 .muted{color:#555} .box{border:2px solid #333;padding:18px} .foot{margin-top:28px;font-size:11px;color:#777}
 .right{text-align:right}
</style>
"""

KG = {
    "holder": "TOKOEV ALIYAR BEKOVICH PE",
    "holder_cyrillic": "ИП ТОКОЕВ АЛИЯР БЕКОВИЧ",
    "number": "20000000000001",
    "address": "KYRGYZ REPUBLIC, BISHKEK, UL. TESTOVAYA 10, APT 5",
    "bank": "Test Bank KG",
    "account": "0000000099",
    "currency": "USD",
    "client": "EXAMPLE CORP INC",
}

US = {
    "holder": "ACME TEST HOLDINGS LLC",
    "holder_statement": "Acme Test Holdings, LLC",
    "number": "99-0000001",
    "address": "123 TEST AVENUE, SUITE 400, AUSTIN, TX 78701",
    "bank": "Example National Bank",
    "account": "000000000123",
    "currency": "USD",
    "client": "SAMPLE CLIENT CORP",
}


def page(body: str) -> str:
    return f'<html><head><meta charset="utf-8">{STYLE}</head><body>{body}</body></html>'


def kg_registration() -> str:
    return page(f"""<div class="box">
<h1>Certificate of state registration of an individual entrepreneur</h1>
<div class="muted">Свидетельство о государственной регистрации индивидуального предпринимателя</div>
<h2>Registrant / Регистрируемое лицо</h2>
<table>
<tr><th>Name (Latin)</th><td>{KG['holder']}</td></tr>
<tr><th>ФИО</th><td>{KG['holder_cyrillic']}</td></tr>
<tr><th>Tax identification number</th><td>{KG['number']}</td></tr>
<tr><th>Registration date</th><td>2023-03-14</td></tr>
<tr><th>Registration authority</th><td>Test Registry Office No. 1</td></tr>
<tr><th>Address</th><td>{KG['address']}</td></tr>
<tr><th>Activity</th><td>Software development (code 62.01)</td></tr>
</table>
<div class="foot">{FOOTER}</div></div>""")


def us_formation() -> str:
    return page(f"""<div class="box">
<h1>Certificate of Formation</h1>
<div class="muted">Limited Liability Company (synthetic specimen)</div>
<h2>Filing entity</h2>
<table>
<tr><th>Entity name</th><td>{US['holder']}</td></tr>
<tr><th>Entity type</th><td>Limited Liability Company</td></tr>
<tr><th>State of formation</th><td>Texas</td></tr>
<tr><th>File number</th><td>0000000001</td></tr>
<tr><th>Date of filing</th><td>2022-06-15</td></tr>
<tr><th>Principal office</th><td>{US['address']}</td></tr>
<tr><th>Registered agent</th><td>Example Agent Services Inc.</td></tr>
</table>
<div class="foot">{FOOTER}</div></div>""")


def us_ein_letter() -> str:
    notice = dt.date.today() - dt.timedelta(days=40)
    return page(f"""<div class="box">
<h1>EIN Assignment Letter</h1>
<div class="muted">Employer identification number notice (synthetic specimen, not issued by any tax authority)</div>
<h2>Entity</h2>
<table>
<tr><th>Entity name</th><td>{US['holder']}</td></tr>
<tr><th>Employer identification number (EIN)</th><td>{US['number']}</td></tr>
<tr><th>Mailing address</th><td>{US['address']}</td></tr>
<tr><th>Date of notice</th><td>{notice}</td></tr>
</table>
<div class="foot">{FOOTER}</div></div>""")


def months_back(end: dt.date, months: int) -> dt.date:
    year, month = end.year, end.month - (months - 1)
    while month < 1:
        month += 12
        year -= 1
    return dt.date(year, month, 1)


def statement(profile: dict, holder: str, end: dt.date, months: int, tag: str, number_label: str) -> str:
    start = months_back(end, months)
    rows = []
    balance = 4200.00
    n = 1
    cursor = start
    while cursor <= end:
        incoming = 5100.00 + 100 * (n % 3)
        balance += incoming
        rows.append((n, cursor.replace(day=5), "Incoming transfer", f"{profile['client']} / {holder}", incoming, balance))
        n += 1
        balance -= 12.00
        rows.append((n, cursor.replace(day=28), "Monthly maintenance fee", holder, -12.00, balance))
        n += 1
        cursor = (cursor.replace(day=1) + dt.timedelta(days=32)).replace(day=1)
    body = "".join(
        f"<tr><td>{i}</td><td>{d}</td><td>{t}</td><td>{p}</td><td class='right'>{a:,.2f}</td><td class='right'>{b:,.2f}</td></tr>"
        for i, d, t, p, a, b in rows
    )
    return page(f"""<h1>{profile['bank']} - Account statement</h1>
<table>
<tr><th>Account holder</th><td>{holder}</td><th>Statement date</th><td>{end + dt.timedelta(days=1)}</td></tr>
<tr><th>{number_label}</th><td>{profile['number']}</td><th>Statement period</th><td>{start} - {end}</td></tr>
<tr><th>Account number</th><td>{profile['account']}</td><th>Currency</th><td>{profile['currency']}</td></tr>
<tr><th>Address</th><td colspan="3">{profile['address']}</td></tr>
</table>
<h2>Transactions</h2>
<table><tr><th>#</th><th>Date</th><th>Type</th><th>Counterparty</th><th>Amount</th><th>Balance</th></tr>{body}</table>
<div class="foot">{FOOTER} ({tag})</div>""")


def us_utility_bill() -> str:
    today = dt.date.today()
    issued = today - dt.timedelta(days=15)
    start = (issued.replace(day=1) - dt.timedelta(days=1)).replace(day=1)
    end = issued.replace(day=1) - dt.timedelta(days=1)
    return page(f"""<h1>Example Electric Co. - Monthly bill</h1>
<table>
<tr><th>Bill to</th><td>{US['holder']}</td><th>Bill date</th><td>{issued}</td></tr>
<tr><th>Service address</th><td colspan="3">{US['address']}</td></tr>
<tr><th>Account number</th><td>00-000-0001</td><th>Service period</th><td>{start} - {end}</td></tr>
</table>
<h2>Charges</h2>
<table>
<tr><th>Description</th><th class="right">Amount (USD)</th></tr>
<tr><td>Energy charge, 1,240 kWh</td><td class="right">148.80</td></tr>
<tr><td>Delivery charge</td><td class="right">36.20</td></tr>
<tr><td>Taxes and fees</td><td class="right">9.45</td></tr>
<tr><th>Total due</th><th class="right">194.45</th></tr>
</table>
<div class="foot">{FOOTER}</div>""")


def to_pdf(name: str, html: str) -> None:
    src = OUT / f"{name}.html"
    src.write_text(html, encoding="utf-8")
    pdf = OUT / f"{name}.pdf"
    subprocess.run(
        [CHROME, "--headless=new", "--disable-gpu", "--no-pdf-header-footer", f"--print-to-pdf={pdf}", src.as_uri()],
        check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=60,
    )
    src.unlink()
    print("wrote", pdf.relative_to(ROOT.parent))


def main() -> int:
    today = dt.date.today()
    recent_end = today.replace(day=1) - dt.timedelta(days=1)
    stale_end = dt.date(2025, 8, 31)

    to_pdf("registration_certificate", kg_registration())
    to_pdf("bank_statement_recent", statement(KG, KG["holder"], recent_end, 3, "recent", "Tax number"))
    to_pdf("bank_statement_stale", statement(KG, KG["holder"], stale_end, 3, "stale", "Tax number"))

    to_pdf("us_certificate_of_formation", us_formation())
    to_pdf("us_ein_letter", us_ein_letter())
    to_pdf("us_bank_statement_recent", statement(US, US["holder_statement"], recent_end, 3, "recent", "EIN"))
    to_pdf("us_bank_statement_stale", statement(US, US["holder_statement"], stale_end, 3, "stale", "EIN"))
    to_pdf("us_utility_bill", us_utility_bill())
    return 0


if __name__ == "__main__":
    sys.exit(main())
