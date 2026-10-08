package dev.evvie.waylandcraft.mixin;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import dev.evvie.waylandcraft.WaylandCraft;
import com.mojang.blaze3d.platform.InputConstants;
import net.minecraft.client.KeyboardHandler;
import net.minecraft.client.Minecraft;
import net.minecraft.client.input.KeyEvent;

@Mixin(KeyboardHandler.class)
public class KeyboardHandlerMixin {
	
	// Capture at the method boundary. Cancelling later lets 26.3 update
	// vanilla KeyMapping state before our handler runs, which leaves keys
	// logically held after input has been redirected to a Wayland window.
	@Inject(method = "keyPress", at = @At("HEAD"), cancellable = true)
	public void onPress(long windowHandle, int action, KeyEvent event, CallbackInfo info) {
		int scancode = WaylandCraft.correctScancode(event.key());

		if(WaylandCraft.instance.bridge != null && (action == InputConstants.PRESS || action == InputConstants.RELEASE)) {
			WaylandCraft.instance.bridge.internalKeyUpdate(scancode, action == InputConstants.PRESS);
		}

		if(Minecraft.getInstance().level == null) return;
		if(Minecraft.getInstance().gui.screen() != null) return;

		if(WaylandCraft.instance.onKeyPress(windowHandle, event.key(), scancode, action, event.modifiers())) info.cancel();
	}
	
}
