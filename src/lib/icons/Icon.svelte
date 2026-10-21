<script lang="ts">
  import type { IconName } from './icon-names';
  import { iconSvg } from './icons';

  interface Props {
    name: IconName;
    /** Zusätzliche Klassen, z. B. zur Größe: `h-8 w-8`. Standard ist 1.5rem. */
    class?: string;
    /** Zugänglicher Name; ohne Angabe ist das Icon rein dekorativ. */
    label?: string;
  }

  let { name, class: className = 'h-6 w-6', label }: Props = $props();
</script>

<!-- eslint-disable-next-line svelte/no-at-html-tags -- SVG stammt aus den eigenen, geprüften Dateien unter src/lib/icons -->
<span
  class="inline-flex shrink-0 items-center justify-center [&>svg]:h-full [&>svg]:w-full {className}"
  role={label ? 'img' : undefined}
  aria-label={label}
  aria-hidden={label ? undefined : 'true'}
>
  {@html iconSvg(name)}
</span>
