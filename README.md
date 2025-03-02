Prénom: Sanndy
Nom : MANIJEAN
Classe : 4e SI Janvier 2025
'
Matière programmation Rust Systeme


Consigne du CC pour avant le 27 Mars 2025: 
● Git repo, add me as contributor

● The project is in no_std

● Commit are looked at, do not commit everything in one time, else it’s
considered cheating

●If you take code from somewhere it has to be credited,else considered
cheating as well

● Code quality (miri/mirai/fuzzers, other cargo utils ….) are bonus but testing
is MANDATORY

● Unsafe must be thoroughly documented using rustdoc safety part

●Comment your code and use rust doc, code exemple are
appreciated for the allocation library

●test your program you can use test_runner 

● No group, is to be done individually

● A report with your design choice is needed, slab allocators can
be a hard thing to do, so I need to understand what you wanted
to do in the first place


Consigne pour l'examen Final : 
●
If you have time (it’s adviced to do so)
–In the second exam you’ll have to implement a FAT32 filesystem
–You can start implementing a no_std compatible FAT32 parser
–Won’t be taken in account for THIS exam, but will help you go faster
for part II


On va devoir utilisé notre allocateur de mémoire coder, pour notre examen final.
------------------------------------------------------------------------------------

1ere Etape: Comment fonctionne un allocateur de mémoire.
Un allocateur de mémoire, c'est un système avec des algorithmes vont réserver de la mémoire dynamiquement pour un programme informatique.

Lorsqu'un programme a besoin de mémoire, voila ce qui se passe:
- Le programme demande à l'allocateur de lui alloué de l'espace en ram
- L'allocateur trouve l'espace libre
- L'allocateur réserve l'espace et renvoie un pointeur au programme, vers cette espace
- Le programme utilise cette espace pour stocket ses données
- En fonction des cas, l'allocateur peut libéré cette dé-allouer l'esace quand il n'est plus utilisé.

Sauf qu'en Rust, avec no_std, ça doit être fait manuellement.


Les types d'allocateurs sont des techniques pour allouer de la mémoire selon le besoin:
- Bump Allocator 
- Free List Allocator
- Buddy Allocator
- Slab Allocator






