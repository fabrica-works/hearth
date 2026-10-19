//! Debian's base-passwd master files (`/usr/share/base-passwd/`, copied as bytes).
pub const PASSWD_MASTER: &[u8] = b"\
root:*:0:0:root:/root:/bin/bash\n\
daemon:*:1:1:daemon:/usr/sbin:/usr/sbin/nologin\n\
bin:*:2:2:bin:/bin:/usr/sbin/nologin\n\
sys:*:3:3:sys:/dev:/usr/sbin/nologin\n\
sync:*:4:65534:sync:/bin:/bin/sync\n\
games:*:5:60:games:/usr/games:/usr/sbin/nologin\n\
man:*:6:12:man:/var/cache/man:/usr/sbin/nologin\n\
lp:*:7:7:lp:/var/spool/lpd:/usr/sbin/nologin\n\
mail:*:8:8:mail:/var/mail:/usr/sbin/nologin\n\
news:*:9:9:news:/var/spool/news:/usr/sbin/nologin\n\
uucp:*:10:10:uucp:/var/spool/uucp:/usr/sbin/nologin\n\
proxy:*:13:13:proxy:/bin:/usr/sbin/nologin\n\
www-data:*:33:33:www-data:/var/www:/usr/sbin/nologin\n\
backup:*:34:34:backup:/var/backups:/usr/sbin/nologin\n\
list:*:38:38:Mailing List Manager:/var/list:/usr/sbin/nologin\n\
irc:*:39:39:ircd:/run/ircd:/usr/sbin/nologin\n\
_apt:*:42:65534::/nonexistent:/usr/sbin/nologin\n\
nobody:*:65534:65534:nobody:/nonexistent:/usr/sbin/nologin\n";
/// The user names with their uid and gid, in the file's order.
pub const PASSWD_IDS: &[(&[u8], u32, u32)] = &[
    (b"root", 0, 0),
    (b"daemon", 1, 1),
    (b"bin", 2, 2),
    (b"sys", 3, 3),
    (b"sync", 4, 65534),
    (b"games", 5, 60),
    (b"man", 6, 12),
    (b"lp", 7, 7),
    (b"mail", 8, 8),
    (b"news", 9, 9),
    (b"uucp", 10, 10),
    (b"proxy", 13, 13),
    (b"www-data", 33, 33),
    (b"backup", 34, 34),
    (b"list", 38, 38),
    (b"irc", 39, 39),
    (b"_apt", 42, 65534),
    (b"nobody", 65534, 65534),
];
pub const GROUP_MASTER: &[u8] = b"\
root:*:0:\n\
daemon:*:1:\n\
bin:*:2:\n\
sys:*:3:\n\
adm:*:4:\n\
tty:*:5:\n\
disk:*:6:\n\
lp:*:7:\n\
mail:*:8:\n\
news:*:9:\n\
uucp:*:10:\n\
man:*:12:\n\
proxy:*:13:\n\
kmem:*:15:\n\
dialout:*:20:\n\
fax:*:21:\n\
voice:*:22:\n\
cdrom:*:24:\n\
floppy:*:25:\n\
tape:*:26:\n\
sudo:*:27:\n\
audio:*:29:\n\
dip:*:30:\n\
www-data:*:33:\n\
backup:*:34:\n\
operator:*:37:\n\
list:*:38:\n\
irc:*:39:\n\
src:*:40:\n\
shadow:*:42:\n\
utmp:*:43:\n\
video:*:44:\n\
sasl:*:45:\n\
plugdev:*:46:\n\
staff:*:50:\n\
games:*:60:\n\
users:*:100:\n\
nogroup:*:65534:\n";
/// The group names with their gid, in the file's order.
pub const GROUP_IDS: &[(&[u8], u32)] = &[
    (b"root", 0),
    (b"daemon", 1),
    (b"bin", 2),
    (b"sys", 3),
    (b"adm", 4),
    (b"tty", 5),
    (b"disk", 6),
    (b"lp", 7),
    (b"mail", 8),
    (b"news", 9),
    (b"uucp", 10),
    (b"man", 12),
    (b"proxy", 13),
    (b"kmem", 15),
    (b"dialout", 20),
    (b"fax", 21),
    (b"voice", 22),
    (b"cdrom", 24),
    (b"floppy", 25),
    (b"tape", 26),
    (b"sudo", 27),
    (b"audio", 29),
    (b"dip", 30),
    (b"www-data", 33),
    (b"backup", 34),
    (b"operator", 37),
    (b"list", 38),
    (b"irc", 39),
    (b"src", 40),
    (b"shadow", 42),
    (b"utmp", 43),
    (b"video", 44),
    (b"sasl", 45),
    (b"plugdev", 46),
    (b"staff", 50),
    (b"games", 60),
    (b"users", 100),
    (b"nogroup", 65534),
];
