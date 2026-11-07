"""Build the independently authored CHM using Microsoft's HTML Help compiler.

Run: python tools/build_help.py --compiler .tools/htmlhelp/compiler/hhc.exe
The generated CHM is checked in; ordinary Cargo builds do not require HHC.
"""
from pathlib import Path
import argparse, contextlib, html, math, re, subprocess

ROOT = Path(__file__).resolve().parents[1]

@contextlib.contextmanager
def compiler_registration(compiler):
    """Temporarily expose a portable compiler's COM classes for this build.

    Existing registrations are untouched. Only newly created per-user keys are
    removed afterwards; Windows runtime classes and files are never replaced.
    """
    import winreg
    classes=['4662DAA2-D393-11D0-9A56-00C04FB68BF7',
             '4662DAA3-D393-11D0-9A56-00C04FB68BF7',
             '4662DAA4-D393-11D0-9A56-00C04FB68BF7',
             '8FA0D5A9-DEDF-11D0-9A61-00C04FB68BF7',
             '8FA0D5AA-DEDF-11D0-9A61-00C04FB68BF7',
             '8FA0D5AB-DEDF-11D0-9A61-00C04FB68BF7']
    view=winreg.KEY_WOW64_32KEY
    created=[]
    prog_created=[]
    try:
        for guid in classes:
            relative='CLSID\\{'+guid+'}'
            try:
                with winreg.OpenKey(winreg.HKEY_CLASSES_ROOT,relative+'\\InprocServer32',0,winreg.KEY_READ|view):
                    continue
            except FileNotFoundError:
                pass
            parent='Software\\Classes\\'+relative
            try:
                with winreg.OpenKey(winreg.HKEY_CURRENT_USER,parent,0,winreg.KEY_READ|view):
                    raise RuntimeError('An incomplete compiler registration already exists: '+guid)
            except FileNotFoundError:
                pass
            created.append(parent)
            with winreg.CreateKeyEx(winreg.HKEY_CURRENT_USER,parent+'\\InprocServer32',0,winreg.KEY_WRITE|view) as key:
                winreg.SetValueEx(key,None,0,winreg.REG_SZ,str(Path(compiler).resolve().with_name('itcc.dll')))
                winreg.SetValueEx(key,'ThreadingModel',0,winreg.REG_SZ,'Both')
        for name,guid in zip(['CmdInt','SvMgr','GroupBuild','SystemWordSink','FTIBuild','WWFilterBuild'],classes):
            for suffix in ['', '.4']:
                relative='ITIR.'+name+suffix
                try:
                    with winreg.OpenKey(winreg.HKEY_CLASSES_ROOT,relative,0,winreg.KEY_READ|view):
                        continue
                except FileNotFoundError:
                    pass
                parent='Software\\Classes\\'+relative
                prog_created.append(parent)
                with winreg.CreateKeyEx(winreg.HKEY_CURRENT_USER,parent,0,winreg.KEY_WRITE|view) as key:
                    winreg.SetValueEx(key,None,0,winreg.REG_SZ,'')
                with winreg.CreateKeyEx(winreg.HKEY_CURRENT_USER,parent+'\\CLSID',0,winreg.KEY_WRITE|view) as key:
                    winreg.SetValueEx(key,None,0,winreg.REG_SZ,'{'+guid+'}')
        yield
    finally:
        for parent in reversed(prog_created):
            winreg.DeleteKeyEx(winreg.HKEY_CURRENT_USER,parent+'\\CLSID',view,0)
            winreg.DeleteKeyEx(winreg.HKEY_CURRENT_USER,parent,view,0)
        for parent in reversed(created):
            winreg.DeleteKeyEx(winreg.HKEY_CURRENT_USER,parent+'\\InprocServer32',view,0)
            winreg.DeleteKeyEx(winreg.HKEY_CURRENT_USER,parent,view,0)


def build(compiler):
    target = ROOT / 'target' / 'help'
    target.mkdir(parents=True, exist_ok=True)
    source = (ROOT / 'help' / 'SmithHelp.md').read_text(encoding='utf-8')
    topics = []
    for title, text in re.findall(r'^## ([^\n]+)\n(.*?)(?=^## |\Z)', source, re.M | re.S):
        number = title.split(' ', 1)[0]
        topics.append((number, title, text.strip()))
    for i, (number, title, text) in enumerate(topics):
        paragraphs = []
        for p in re.split(r'\n\s*\n', text):
            if p.startswith('> '):
                paragraphs.append('<pre>'+html.escape(p.replace('> ', ''))+'</pre>')
            else:
                paragraphs.append('<p>'+html.escape(p.replace('\n', ' '))+'</p>')
        links = []
        if i: links.append(f'<a href="topic{i}.htm">Previous</a>')
        links.append('<a href="topic1.htm">Home</a>')
        if i+1<len(topics): links.append(f'<a href="topic{i+2}.htm">Next</a>')
        body = ''.join(paragraphs)
        if i == 0: body = '<img src="cover.png" width="411" height="390" alt="Smith chart">'+body
        page = f'''<!doctype html><html><head><meta http-equiv="Content-Type" content="text/html; charset=windows-1252"><title>{html.escape(title)}</title>
<style>body{{font:10pt Arial;margin:0;background:white;color:black}}h1{{font:bold 9.5pt Verdana;background:#99c8ff;margin:0;padding:4px}}.body{{padding:8px}}p{{line-height:1.4}}pre{{white-space:pre-wrap;background:#f3f3f3;padding:8px;font:9pt Consolas}}nav{{padding:4px;background:#eee;font:8pt Arial}}nav a{{margin-right:14px}}footer{{border-top:1px solid #aaa;padding:4px;font:8pt Arial}}img{{display:block;margin:8px auto}}</style></head>
<body><h1>{html.escape(title)}</h1><nav>{' '.join(links)}</nav><div class="body">{body}</div><footer>SmithCraft — independent Rust implementation</footer></body></html>'''
        (target/f'topic{i+1}.htm').write_bytes(page.encode('ascii',errors='xmlcharrefreplace'))
    contents = ['<!doctype html><html><body><ul>']
    depth = 0
    for i, (number, title, _) in enumerate(topics):
        level = number.count('.')
        while depth < level: contents.append('<ul>'); depth += 1
        while depth > level: contents.append('</ul>'); depth -= 1
        contents.append(f'<li><object type="text/sitemap"><param name="Name" value="{html.escape(title)}"><param name="Local" value="topic{i+1}.htm"></object>')
    contents.extend(['</ul>']*(depth+1));contents.append('</body></html>')
    (target/'contents.hhc').write_text('\n'.join(contents),encoding='utf-8')
    index=['<!doctype html><html><body><ul>']
    for title,i in sorted((t[1].split(' ',1)[1],i) for i,t in enumerate(topics)):
        index.append(f'<li><object type="text/sitemap"><param name="Name" value="{html.escape(title)}"><param name="Local" value="topic{i+1}.htm"></object>')
    index.append('</ul></body></html>');(target/'index.hhk').write_text('\n'.join(index),encoding='utf-8')
    # The cover is a newly drawn mathematical diagram, not an image from Smith.
    from PIL import Image, ImageDraw, ImageFont
    im=Image.new('RGB',(600,600),'white');d=ImageDraw.Draw(im)
    point=lambda z:(300+z.real*260,300-z.imag*260)
    for sign,color in [(1,(255,90,90)),(-1,(90,170,245))]:
        for r in [.2,.5,1,2,4,10]:
            points=[point(sign*(r/(1+r)+complex(math.cos(a),math.sin(a))/(1+r))) for a in [2*math.pi*k/500 for k in range(501)]]
            d.line(points,fill=color,width=1)
        for x in [-4,-2,-1,-.5,-.2,.2,.5,1,2,4]:
            values=[40*(k/300)**3 for k in range(301)]
            points=[point(sign*(complex(r,x)-1)/(complex(r,x)+1)) for r in values];d.line(points,fill=color,width=1)
    d.ellipse((40,40,560,560),outline='black',width=2)
    font=ImageFont.truetype('C:/Windows/Fonts/arial.ttf',42)
    d.text((300,440),'Circuit Design with',font=font,fill='blue',anchor='mm');d.text((300,490),'Smith Chart',font=font,fill='blue',anchor='mm')
    im.save(target/'cover.png')
    destination=ROOT/'assets'/'SmithHelp.chm';destination.parent.mkdir(exist_ok=True)
    files='\n'.join(f'topic{i+1}.htm' for i in range(len(topics)))+'\ncover.png'
    project=f'''[OPTIONS]
Compatibility=1.1 or later
Compiled file={destination}
Contents file=contents.hhc
Index file=index.hhk
Default topic=topic1.htm
Default Window=main
Display compile progress=No
Full-text search=Yes
Language=0x409 English (United States)
Title=Help V4.1 — Rust
[WINDOWS]
main="Help V4.1 - Rust","contents.hhc","index.hhk","topic1.htm","topic1.htm",,,,,0x61520,260,0x387e,[5,5,790,560],,,,,,,0
[FILES]
{files}
'''
    (target/'help.hhp').write_text(project,encoding='utf-8')
    with compiler_registration(compiler):
        result=subprocess.run([str(Path(compiler).resolve()),str(target/'help.hhp')],cwd=target,capture_output=True)
    print(result.stdout.decode('cp1252',errors='replace'))
    if not destination.exists() or destination.read_bytes()[:4]!=b'ITSF':
        raise RuntimeError('HHC did not produce a compiled help file')
    if b'HHC6003' in result.stdout:
        raise RuntimeError('The compiler could not build the full-text search index')
    if b'$FIftiMain' not in destination.read_bytes():
        raise RuntimeError('The compiled help does not contain its full-text search index')
    print(f'Built {len(topics)} help topics: {destination} ({destination.stat().st_size} bytes)')

if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--compiler',required=True);build(parser.parse_args().compiler)
