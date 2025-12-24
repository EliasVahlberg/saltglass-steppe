# CRITICAL RENDERING ISSUE

## Problem
There is a major issue in the lighting renderer that was causing torch lighting to disappear after equipping and ending turn. 

## Root Cause Analysis
- Visual effects system was overriding lighting-dimmed colors
- Base entity colors (enemies, NPCs, items) were not applying lighting
- Equipment lighting detection had conditional logic that could fail
- Light map was not being recalculated after save/load

## Temporary Fix Applied
- Always provide default player light (intensity 100, radius 5)
- Applied dim_color() to all entity rendering
- Fixed visual effects to respect lighting

## Outstanding Issues
The renderer still has fundamental problems that need investigation:
1. Equipment light source detection may be unreliable
2. Light calculation or application may have edge cases
3. Turn-based lighting updates may have timing issues
4. Visual effects integration with lighting needs review

## Priority: HIGH
This affects core gameplay visibility and needs thorough investigation of the entire lighting pipeline from calculation to rendering.

## Date: 2025-12-22
## Status: PARTIALLY RESOLVED - NEEDS FURTHER INVESTIGATION
