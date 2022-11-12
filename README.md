# Rlox
## A toy programing language written in Rust.

This is my implementation of the bytecode VM from the book [Crafting Interpreters](http://craftinginterpreters.com/ "crafting interpreters").

## Usage
### Data Types
    1.0
    true / false
    "string"
    "concat " + "string"
### Variables
```
// Declare variable.
var n;
// Assignment
n = 8;
var m = 7;
// Prints 15
print m + n;
```
### Loops
#### For Loops
```
for (var i = 0; i < 100; i = i + 1) {
  print i;
}
```
#### While Loops
```
var i = 0;
while (i < 100) {
  print i; 
  i = i + 1;
}
```
### Functions
```
fun addOne(n) {
  return n + 1;
}
// Prints 4.
print addOne(3);
```
### Classes
#### Definitions
```
class Animal {
    setSpecies(name) {
        this.species = name;         
    }
}
```
#### Methods and variables
```
var n = Animal();
n.setSpecies("dog");
print n.species;
n.coatColor = "black";
```
#### Init function
```
class Animal {
  init(species, coatColor) {
    this.species = species;
    this.coatColor = coatColor;
  }
}
var m = Animal("dog", "black");
print m.species;
print m.coatColor;
```
#### Inheritance
```
class Dog < Animal {
    bark() {
      print "the " + this.species + " barks.";
    }
}
```
