package dev.evvie.waylandcraft.mixin;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import dev.evvie.waylandcraft.WaylandCraft;
import dev.evvie.waylandcraft.render.WindowTranslucencyHotfix;
import net.minecraft.client.Minecraft;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;
import net.minecraft.world.phys.BlockHitResult;
import net.minecraft.world.phys.HitResult;
import net.minecraft.world.phys.Vec3;

@Mixin(Minecraft.class)
public class MinecraftMixin {
	
	// 26.3 renamed the old "Render" profiler/error-section markers. Inject at
	// stable method boundaries rather than into version-specific marker strings.
	@Inject(method = "runTick", at = @At("HEAD"))
	public void updateRunTick(boolean doTick, CallbackInfo info) {
		WaylandCraft.instance.update();
	}
	
	@Inject(method = "renderFrame", at = @At("TAIL"))
	public void hotfixRenderFrame(boolean advanceGameTime, CallbackInfo info) {
		WindowTranslucencyHotfix.render();
	}
	
	@Inject(method = "pick", at = @At("TAIL"))
	public void pick(float partialTicks, CallbackInfo info) {
		HitResult result = Minecraft.getInstance().hitResult;
		Vec3 pos = Minecraft.getInstance().player.getEyePosition(partialTicks);
		
		WaylandCraft.instance.trueGameHitResult = result;
		
		WaylandCraft.instance.updatePointer();
		
		if(WaylandCraft.instance.overridePickBlock) {
			Minecraft.getInstance().hitResult = BlockHitResult.miss(pos, Direction.DOWN, BlockPos.containing(pos));
			Minecraft.getInstance().crosshairPickEntity = null;
		}
	}
	
}
