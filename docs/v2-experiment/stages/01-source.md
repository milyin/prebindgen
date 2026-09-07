<!-- spec: {"kind": "stage", "stage": "01-source"} -->

# Capture source items

[Experiment contents](../README.md)

## Input

The annotated source crate supplies captured definitions. Capture identifies source items before any foreign representation is selected.

## Operation and owner

The source capture mechanism records marked functions and type declarations, preserving source order, source module and diagnostic locations. A parameter or field belongs to its containing item; it does not become another top-level capture or exported binding.

## Output and failure contract

Captured item definitions are the input to Flat. Capture does not select names for foreign declarations, converters or JNI methods. Malformed capture data is an input error; a source form that Flat cannot model is handled during model construction.

## Apply this stage

- [Function: stamp_sum](../examples/function/01-source.md)
- [Struct: Stamp](../examples/struct/01-source.md)

## Pipeline navigation

[Next](02-flat.md)
