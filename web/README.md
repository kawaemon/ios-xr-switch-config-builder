# ncs l2 config builder

ncs5502 (ios xr) は management switch として機能できるものの、config の書き方が非常にややこしい。このツールはそれを支援する機能を提供する。

ncs5502 では config を以下のように書く。

```
interface FortyGigE0/0/0/46
  description To:eth1.server1
  mru 9216
interface FortyGigE0/0/0/46 l2transport
  description servers,To:eth1.server1
  encapsulation dot1q 300
  rewrite ingress tag pop 1 symmetric

interface BVI300
  description servers
  ipv4 address 192.168.1.1 255.255.255.0

l2vpn
  bridge group VLAN
    bridge-domain VLAN300
      description servers
      interface FortyGigE0/0/0/46.300
      routed interface BVI300
```

これは一般的なマネジメントスイッチに比べると非常に冗長で、特に vlan 数が多くなればなるほど config 管理が難しくなる。
一般的な Cisco 構文ライクなスイッチでは以下のように書ける。

```
vlan database
  vlan 300 name servers

interface FortyGigE0/0/0/46
  description To:eth1.server1
  mru 9216
  switchport mode trunk
  switchport trunk allowed vlan add 300

interface BVI300
  description servers
  ipv4 address 192.168.1.1 255.255.255.0
```

また変更を加えるには

```
vlan database
  vlan 301 name servers-2
  vlan 500 name mgmt

interface FortyGigE0/0/0/46
  switchport trunk allowed vlan add 301 500
  switchport trunk allowed vlan remove 300
```

のようにコマンドを書いていく。このツールは、後者のような構文を入力することで、前者の設定変更を行う入力をコピペ可能な形式で自動生成するツールである。
先ほどの例では、以下のようなテキストが生成される。

```
no interface FortyGigE0/0/0/46.300 l2transport
interface FortyGigE0/0/0/46.301 l2transport
  description servers-2,To:eth1.server1
  encapsulation dot1q 301
  rewrite ingress tag pop 1 symmetric
interface FortyGigE0/0/0/46.500 l2transport
  description mgmt,To:eth1.server1
  encapsulation dot1q 500
  rewrite ingress tag pop 1 symmetric

l2vpn
  bridge group VLAN
    bridge-domain VLAN300
      no interface FortyGigE0/0/0/46.300
    exit
    bridge-domain VLAN301
      description servers-2
      interface FortyGigE0/0/0/46.301
    exit
    bridge-domain VLAN500
      description mgmt
      interface FortyGigE0/0/0/46.500
    exit
  exit
exit
```
