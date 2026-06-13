# Agent Communication Guidelines

## Tone & Style

When writing documentation, release notes, or any user-facing content:

- **Direct and concise** - no marketing fluff
- **No excessive emojis** - use sparingly or not at all
- **No AI-like phrases** - avoid "exciting", "amazing", "comprehensive", etc.
- **Short sentences** - get to the point
- **Factual** - state what it does, not why it's great
- **Technical when needed** - don't oversimplify complex features

### Examples

❌ **Too verbose/AI-like:**
> "🎯 PRIMARY: Orphaned Mods Cleanup (Major space savings!)
> 
> **The Problem:** You tried 5 modlists, kept 2, deleted 3. But those deleted modlists' mods are **still in your downloads folder wasting space!**
> 
> **The Solution:** This amazing tool compares your mod files against your **active modlists**..."

✅ **Direct/tok:**
> **PRIMARY: Orphaned Mods Cleanup
> 
> Removes mods from deleted/inactive modlists. Compares your files against active modlists using `.wabbajack` files."

### Specific Rules

1. **No specific size predictions** - Space savings vary by user
   - ❌ "50-150 GB typical savings"
   - ✅ "Space savings depend on number of modlists and mods"
   
2. **Don't oversimplify technical features** - Be accurate
   - ❌ "Tool uses timestamps only"
   - ✅ Mention actual complexity if feature has multiple checks/parameters

3. **Remove marketing language**
   - ❌ "Amazing", "Powerful", "Revolutionary", "Game-changing"
   - ✅ Just describe what it does

4. **Keep warnings direct**
   - ❌ "⚠️ CRITICAL: You **MUST** have..."
   - ✅ "Requires: .wabbajack files for all active modlists"

5. **Bullet points over paragraphs** - Easier to scan

6. **Code/technical terms in backticks** - `filename.ext`, `.wabbajack`

## Commit Messages

Keep them short and descriptive:
- ✅ "Fix loop variable capture bugs"
- ✅ "Simplify CHANGELOG format"
- ❌ "Fixed several critical issues with the UI threading model that were causing performance problems"

### Author Configuration
- **IMPORTANT**: All commits and tags must be authored under your GitHub name and email: `Berkay Yetgin <85676216+Yakrel@users.noreply.github.com>`.
- Never commit as `root` or other default system users. Always run `git config user.name "Berkay Yetgin"` and `git config user.email "85676216+Yakrel@users.noreply.github.com"` locally in the repository before making commits.
