# monokakido_to_mdict
a multi-thread parser for creating mdict dictionaries from the monokakido data

### warning:
this library is in development, if you're going to use it, pick [this commit](https://github.com/KonstantinDjairo/monokakido_to_mdict/tree/da3c4fe685641acf29f52cdb10639e429b75b0ad)

## Usage :
you need to process the desired dictionary with [this cli](https://git.ajattix.org/hashirama/mkd-utils)\
and then pass the **contents** folder as INPUT_DIR : <br></br>
```shell
parser_to_mdict [--headword-tag TAG] INPUT_DIR
```
the output is a text file ready to be passed to a mdict processor, like [mdict-utils](https://github.com/liuyug/mdict-utils) <br></br>
example of a headword from 白水社 現代ポルトガル語辞典 :
```
habitualidade
<html xmlns="http://www.w3.org/1999/xhtml" lang="ja"><head><meta http-equiv="Content-Type" content="text/html;charset=utf-8"/><meta name="viewport" content="width=device-width, initial-scale = 1.0, user-scalable = yes, minimum-scale=0.333, maximum-scale=3.0"/><link rel="stylesheet" href="HSS_PT_JA.css" media="all"/></head><body><div xmlns="" class="entry" id="28523">
<div class="mida">habitualidade</div><div id="index"><a href="#28523-0001">女性名詞</a></div><span class="hatsu">［<span class="pron">abituali ́dadi</span>］</span><div class="gogi" id="28523-0001"><span class="hinsi">女</span> <span class="genre genre-region">ブラジル</span> ＝<strong>habitualismo</strong></div>
</div></body></html>
```
after passing that file to mdict-utils, it will be ready for use. <br></br>
![image](https://github.com/user-attachments/assets/0ceb9ef4-db6a-46cd-82c2-7867d2442a0a)\
Enjoy! (⌒ω⌒)
## License
```
This program is © 2025, Hashirama Senju 

This program is published and distributed under the Academic Software License v1.0 (ASL).
```


### useful links
http://web.archive.org/http://goldendict.org/forum/viewtopic.php?f=4&t=6980
