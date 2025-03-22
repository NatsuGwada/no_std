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

------------------------------------------------------------------------------------

1ere Etape: Comment fonctionne un allocateur de mémoire.
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



J'ai choisi de coder sur une VM par sécurité. On va manipuler de la mémoire, pour éviter toutes casse local (corruption de mémoire). C'est mieux. Mais au final c'etait chiant car mon copie paste ne passais pas souvent ce qui oblige a faire autrement mais c'est plus long, donc je suis repasser en local pour mes test.

Le Slab_Allocator est compatible avec un environnement no_std,doncsans accès à la bibliothèque standard de rust.
ça met un context bas niveau ( cool pour du embarqué ou des noyaux car c'est ce qui est demander)
un noyaux(kernel) n'a pas accès a la bibliothèque standard, donc on va utilise le module "code::" aulieu de "std::)

--------------------------------------------------------------------------------------------------------
## Design de l’allocateur

J’ai implémenté un allocateur de type Slab, qui fonctionne en créant des blocs (slabs) de taille fixe.  
4 tailles choisi : 16, 32, 64 et 128 octets.  
Chaque taille a sa propre liste chaînée de blocs libres.

Quand on veut allouer de la mémoire :
- on regarde la taille demandée
- on choisit la taille de bloc qui correspond (ex: 20 octets → bloc de 32)
- on prend un bloc libre dans la liste
- si la liste est vide, on en crée 10 nouveaux

Quand on désalloue :
- on remet le bloc dans la liste des libres

Ce design est rapide, évite la fragmentation et marche très bien pour FAT32.

## Schéma visuel mémoire

Voici à quoi ressemble la mémoire allouée par blocs (liste chaînée) :

[ Bloc libre 1 ] → [ Bloc libre 2 ] → [ Bloc libre 3 ] → null

Chaque bloc contient un pointeur vers le suivant (`next: *mut SlabBlock`)

Quand on alloue un bloc, on prend celui en tête et on avance la liste.
Quand on libère, on met le bloc au début de la liste.

## Sécurité : `unsafe` expliqué

Rust interdit normalement de manipuler la mémoire brute.  
Mais comme on est en `no_std`, on utilise des `unsafe` pour accéder à des pointeurs, ou écrire dans la mémoire.

Chaque fonction `unsafe` est documentée avec `/// # Safety` dans le code, comme demandé.  
On garantit :
- qu’on accède à une mémoire allouée et alignée
- qu’il n’y a pas d’accès concurrent (pas de multithread, car accéder à la mémoire en meme temps ça peut faire bogué). 
- qu’on évite les doublons de libération (`double free`)

C'est quoi un thread? Bin c'est un petit sous programme qui s'exécute en parallèle avec d'autre. par exemple téléchager un fichier , l'action se fait par un thread. c'est un truc qui fait une tâche ou une action.
Ensuite...

## Tests réalisés

Les tests sont dans le fichier `lib.rs` dans le module `#[cfg(test)]`.

J’ai testé :
- l’allocation d’un `u32` et l’écriture dedans ( allocation simple et minimal, pour valider le fonctionnement)
- des allocations avec différentes tailles (`u8`, `u32`, `TestStruct`, etc., pour s'assurer que find_slab_index() fonctionne pour l'allocation adaptative)
- la réutilisation d’un bloc après désallocation (test important ! pour savoir si ça réutilise proprement la mémoire)

Les tests utilisent `unsafe` comme dans le vrai code, mais sont bien encadrés.


Les copier coller les resultats des test dans le fichier  test_et_warning il y a des warning mais c'est normal en no_std
