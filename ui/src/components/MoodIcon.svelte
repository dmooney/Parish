<script lang="ts">
	import {
		Sun,
		Cloud,
		CloudRain,
		CloudLightning,
		Moon,
		MoonStars,
		Flame,
		Flower,
		Heart,
		Eye,
		Lightning,
		Wind,
		Sparkle,
		Cross,
		ShieldCheck,
		SmileyNervous,
		Clover,
		Ghost,
		Leaf,
		Hourglass,
		Sunglasses,
		StarFour
	} from 'phosphor-svelte';

	/** The freeform mood string from the NPC. */
	let { mood, size = '1.1em' }: { mood: string; size?: string } = $props();

	/** Icon lookup table — maps mood keywords to Phosphor component + color. */
	const MOOD_ICONS: Array<{
		keywords: string[];
		icon: typeof Sun;
		color: string;
	}> = [
		// Negative/intense — checked first
		{ keywords: ['angry', 'furious', 'enraged', 'irate'], icon: CloudLightning, color: '#c44' },
		{ keywords: ['afraid', 'fearful', 'terrified', 'scared'], icon: Ghost, color: '#a8a' },
		{ keywords: ['anxious', 'nervous', 'worried', 'uneasy'], icon: SmileyNervous, color: '#b97' },
		{ keywords: ['sad', 'grief', 'mournful', 'sorrowful'], icon: CloudRain, color: '#68a' },
		{
			keywords: ['melanchol', 'wistful', 'nostalgic', 'pensive'],
			icon: MoonStars,
			color: '#88a'
		},
		{
			keywords: ['irritat', 'frustrat', 'annoyed', 'grumpy'],
			icon: Lightning,
			color: '#c84'
		},
		{ keywords: ['suspicious', 'wary', 'distrustful'], icon: Eye, color: '#a88' },

		// Positive
		{
			keywords: ['joy', 'elated', 'ecstatic', 'delighted'],
			icon: Sun,
			color: '#da4'
		},
		{ keywords: ['cheerful', 'jovial', 'merry', 'jolly'], icon: Clover, color: '#6a6' },
		{ keywords: ['friendly', 'welcoming', 'hospitable'], icon: Heart, color: '#c77' },
		{ keywords: ['amused', 'laughing', 'mirthful'], icon: Sparkle, color: '#ca6' },
		{ keywords: ['passionate', 'fervent', 'ardent'], icon: Flame, color: '#c64' },

		// Neutral/cognitive
		{
			keywords: ['contemplat', 'thoughtful', 'reflective', 'ponder'],
			icon: Moon,
			color: '#8a8'
		},
		{ keywords: ['determined', 'resolute', 'steadfast'], icon: ShieldCheck, color: '#8a6' },
		{ keywords: ['alert', 'watchful', 'vigilant', 'attentive'], icon: Eye, color: '#aa8' },
		{ keywords: ['calm', 'serene', 'peaceful', 'tranquil'], icon: Leaf, color: '#6a8' },
		{ keywords: ['content', 'satisfied', 'pleased'], icon: Flower, color: '#a96' },
		{ keywords: ['restless', 'agitated', 'fidgety'], icon: Wind, color: '#8a8' },
		{ keywords: ['tired', 'weary', 'exhausted', 'sleepy'], icon: Hourglass, color: '#888' },
		{
			keywords: ['stoic', 'guarded', 'reserved', 'neutral'],
			icon: Cross,
			color: '#888'
		},
		{ keywords: ['curious', 'intrigued', 'interested'], icon: Sparkle, color: '#8aa' },
		{ keywords: ['shy', 'bashful', 'embarrass'], icon: Flower, color: '#a8a' },
		{ keywords: ['proud', 'smug', 'self-satisfied'], icon: Sunglasses, color: '#a86' },
		{ keywords: ['surprised', 'astonished', 'shocked'], icon: StarFour, color: '#ca8' },
		{ keywords: ['warm'], icon: Heart, color: '#c77' }
	];

	/** Resolve the mood string to an icon entry. */
	function resolve(m: string) {
		const lower = m.toLowerCase();
		for (const entry of MOOD_ICONS) {
			if (entry.keywords.some((kw) => lower.includes(kw))) return entry;
		}
		return { icon: Flower, color: '#a96' }; // fallback: content flower
	}

	let match = $derived(resolve(mood));
</script>

<span class="mood-icon" title={mood}>
	<match.icon weight="thin" {size} color={match.color} />
</span>

<style>
	.mood-icon {
		display: inline-flex;
		align-items: center;
	}
</style>
