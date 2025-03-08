# Rapport sur l'implémentation d'un Slab Allocator en Rust

## Introduction
Ce projet consiste à implémenter un allocateur de mémoire de type "slab" en Rust, qui est compatible avec les environnements no_std. Un slab allocator est particulièrement utile pour l'allocation efficace d'objets de tailles fixes, ce qui est courant dans les systèmes d'exploitation.

## Conception
Mon allocateur est basé sur les principes suivants:

1. **Catégories de tailles** : J'utilise 4 tailles de blocs différentes (16, 32, 64, 128 octets) pour gérer efficacement différentes demandes d'allocation.

2. **Listes chaînées** : Pour chaque taille, je maintiens une liste chaînée de blocs libres, ce qui permet de réutiliser rapidement la mémoire libérée.

3. **Allocation groupée** : J'alloue 10 slabs à la fois pour réduire les frais généraux d'allocation.

4. **Allocation à la demande** : Les blocs sont alloués uniquement lorsqu'ils sont nécessaires, ce qui économise de la mémoire.

## Difficultés rencontrées
Plusieurs défis se sont présentés lors du développement:

- Comprendre le concept de "slab allocation" et comment l'implémenter efficacement
- Gérer correctement les pointeurs bruts en Rust, qui nécessitent des blocs `unsafe`
- Assurer que l'allocateur fonctionne correctement dans un environnement no_std

## Fonctionnement
1. Quand une allocation est demandée, l'allocateur cherche la taille de slab appropriée
2. S'il n'y a pas de bloc libre disponible, il en alloue 10 nouveaux
3. Il prend le premier bloc libre de la liste et le retourne
4. Lors de la désallocation, le bloc est ajouté à la liste des blocs libres

## Conclusion
Cette implémentation d'un slab allocator montre comment on peut gérer efficacement la mémoire dans un environnement sans système d'exploitation. Bien que simple, cet allocateur offre les fonctionnalités de base nécessaires et pourrait être étendu avec des fonctionnalités supplémentaires comme la fusion de blocs adjacents.
