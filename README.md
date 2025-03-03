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


L'objectif plus tard est d'utilisé l'allocateur pour l'imprementé a un systeme de fichier FAT32


Pas de Bump Allocateur: pourquoi? = Il alloue en avançant de pointeur en pointeur vers l'avant uniquement. Pour libérer de la mémoire, il faut tout libéré d'un coup et FAT32 à besoin de libérer des structures mémoire précises individuellement. Il n'est pas adapté.

Pas de Free List Allocator : pourquoi? = L'allocation est fait par liste chainée de bloc libres, Son nombre important de fragmentation fait, qu'il faut a chaque fois cherché des bloc libres de la bonne taille,ça demande trop de gestion de tailles fixes d'objets(entrée répertoires, Cluster de FAT, etc..) et se serait trop lent pour le FileSystem

Pas de Buddy Allocator: pourquoi? = L'allocation mémoire est divisé en bloc de puissance de 2 ( exemple: 1MB -> 512KB -> 256KB -> 128KB.... -> 2KB -> 1KB). Si je veux allouer 32 octets un objet, 64 ou 128 octets seront allouer car Buddy fonctionne en puissance de 2 et c'est du gaspillage. Pour FAT32 on a déja une taille d'objets Fixe

Meilleur choix : Slab Allocator 
pourquoi ? = Il est concu pour les structures de tailles fixe (prépartion de bloc adapté possible)
             Il évite la fragmentation ( Slab stock que un type d'objet)
             Il est rapide (pas de recherche de bloc comme fee list, il prend un bloc disponible)



J'ai choisi de coder sur une VM par sécurité. On va manipuler de la mémoire, pour éviter toutes casse local (corruption de mémoire). C'est mieux.



