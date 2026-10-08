package dev.evvie.waylandcraft.utils;

import java.util.UUID;

import dev.evvie.waylandcraft.item.WindowHandle;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;

/** Dedicated-server-safe helpers kept separate from client rendering utilities. */
public final class ServerWaylandCraftUtils {
	private ServerWaylandCraftUtils() {}

	public static ServerPlayer getPlayer(ServerLevel level, UUID id) {
		for(ServerPlayer player : level.players()) {
			if(WindowHandle.getPlayerUUID(player).equals(id)) return player;
		}
		return null;
	}

	public static boolean isHandleValid(ServerLevel level, WindowHandle handle) {
		if(handle == null) return false;

		ServerPlayer player = getPlayer(level, handle.player());
		return player != null && ((IMyServerPlayer) player).getAliveWindows().contains(handle.handle());
	}
}
