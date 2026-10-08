package dev.evvie.waylandcraft.mixin;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import com.mojang.renderpearl.backend.opengl.GlBackend;

@Mixin(GlBackend.class)
public class GlBackendMixin {
	
	// Minecraft 26.3 creates its OpenGL context through SDL.  EGL setup is now
	// selected by SDL itself, so the old GLFW window-hint injection is obsolete.
	
}
