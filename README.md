# tag-search

tag-search:
    $ tag-search <tag_pp.csv/geotag_pp.csvのあるディレクトリ>
    キャッシュは通常無効なので、有効にするにはビルド時に--with-featuresでcacheを有効にする

    仕様：
        - 8080番のquery.htmlでリクエストを受け付ける
        - クエリパラメータは次の2つ
            - tag   (文字列): タグ
            - cache (真偽値): キャッシュを有効にするか
