#!/usr/bin/env python3
"""Generate synthetic KYB test documents (fictional people, addresses and numbers).

Output goes to fixtures/out/*.pdf. Requires Google Chrome (headless print-to-pdf).
"""
import datetime as dt
import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent
OUT = ROOT / "out"
OUT.mkdir(exist_ok=True)
CHROME = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"

HOLDER_LATIN = "TOKOEV ALIYAR BEKOVICH"
HOLDER_CYR = "ТОКОЕВ АЛИЯР БЕКОВИЧ"
REG_NO = "20000000000001"  # fictional, 14 digits like a Kyrgyz tax id
ADDRESS = "KYRGYZ REPUBLIC, BISHKEK, UL. TESTOVAYA 10, APT 5"
FOOTER = "Synthetic test document. Fictional data. Not a real document."

STYLE = """
<style>
 body{font-family:Arial,Helvetica,sans-serif;margin:40px;color:#111;font-size:13px}
 h1{font-size:20px;margin:0 0 4px} h2{font-size:15px;margin:18px 0 6px}
 table{border-collapse:collapse;width:100%} td,th{border:1px solid #888;padding:4px 6px;text-align:left}
 .muted{color:#555} .box{border:2px solid #333;padding:18px} .foot{margin-top:28px;font-size:11px;color:#777}
</style>
"""


def html_registration() -> str:
    return f"""<html><head><meta charset="utf-8">{STYLE}</head><body><div class="box">
<h1>Certificate of state registration of an individual entrepreneur</h1>
<div class="muted">Свидетельство о государственной регистрации индивидуального предпринимателя</div>
<h2>Registrant / Регистрируемое лицо</h2>
<table>
<tr><th>Name (Latin)</th><td>{HOLDER_LATIN} PE</td></tr>
<tr><th>ФИО</th><td>ИП {HOLDER_CYR}</td></tr>
<tr><th>Tax identification number</th><td>{REG_NO}</td></tr>
<tr><th>Registration date</th><td>2023-03-14</td></tr>
<tr><th>Registration authority</th><td>Test Registry Office No. 1</td></tr>
<tr><th>Address</th><td>{ADDRESS}</td></tr>
<tr><th>Activity</th><td>Software development (code 62.01)</td></tr>
</table>
<div class="foot">{FOOTER}</div></div></body></html>"""


def html_statement(end: dt.date, months: int, tag: str) -> str:
    start = (end.replace(day=1) - dt.timedelta(days=30 * (months - 1))).replace(day=1)
    rows = []
    balance = 4200.00
    cur = start
    n = 1
    while cur <= end:
        incoming = 5100.00 + 100 * (n % 3)
        balance += incoming
        rows.append((n, cur.replace(day=min(cur.day + 4, 28)), "SWIFT Payment Incoming", "EXAMPLE CORP INC / " + HOLDER_LATIN + " PE", "Software services", incoming, balance))
        n += 1
        fee = -3.50
        balance += fee
        rows.append((n, cur.replace(day=28), "Account Maintenance Fee", HOLDER_LATIN + " PE", "Monthly fee", fee, balance))
        n += 1
        cur = (cur.replace(day=1) + dt.timedelta(days=32)).replace(day=1)
    body = "".join(
        f"<tr><td>{i}</td><td>{d}</td><td>{t}</td><td>{p}</td><td>{e}</td><td>{a:,.2f}</td><td>{b:,.2f}</td></tr>"
        for i, d, t, p, e, a, b in rows
    )
    return f"""<html><head><meta charset="utf-8">{STYLE}</head><body>
<h1>Test Bank KG - STATEMENT</h1>
<table>
<tr><th>Name</th><td>{HOLDER_LATIN} PE</td><th>Document creation date</th><td>{end + dt.timedelta(days=1)}</td></tr>
<tr><th>Tax number</th><td>{REG_NO}</td><th>Date interval</th><td>{start} - {end}</td></tr>
<tr><th>Account number</th><td>0000000099</td><th>Currency</th><td>USD</td></tr>
<tr><th>Address</th><td colspan="3">{ADDRESS}</td></tr>
</table>
<h2>Transactions</h2>
<table><tr><th>#</th><th>Date</th><th>Type</th><th>Payer / Beneficiary</th><th>Explanation</th><th>Amount</th><th>Balance</th></tr>{body}</table>
<div class="foot">{FOOTER} ({tag})</div></body></html>"""


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
    recent_end = (today.replace(day=1) - dt.timedelta(days=1))  # end of previous month
    to_pdf("registration_certificate", html_registration())
    to_pdf("bank_statement_recent", html_statement(recent_end, 3, "recent"))
    to_pdf("bank_statement_stale", html_statement(dt.date(2025, 8, 31), 3, "stale"))
    return 0


if __name__ == "__main__":
    sys.exit(main())
